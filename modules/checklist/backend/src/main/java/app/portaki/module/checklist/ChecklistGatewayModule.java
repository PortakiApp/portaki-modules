package app.portaki.module.checklist;

import java.sql.ResultSet;
import java.sql.SQLException;
import java.time.Instant;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.stereotype.Component;

import app.portaki.sdk.gateway.GatewayModuleContext;
import app.portaki.sdk.gateway.PortakiCommandHandler;
import app.portaki.sdk.gateway.PortakiModuleEvent;
import app.portaki.sdk.gateway.PortakiQueryHandler;
import app.portaki.sdk.module.PortakiModule;

@Component
@PortakiModule("checklist")
public class ChecklistGatewayModule {

    private final JdbcTemplate jdbc;

    public ChecklistGatewayModule(JdbcTemplate jdbc) {
        this.jdbc = jdbc;
    }

    @PortakiQueryHandler(value = "checklist.items", scope = "stay:read")
    public List<Map<String, Object>> listItems(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID tenantId = ctx.tenantIdInternal();
        UUID propertyId = UUID.fromString(ctx.propertyId());
        return jdbc.query(
                """
                SELECT id, label_fr, label_en, sort_order
                FROM t_e_module_checklist_items
                WHERE tenant_id = ? AND property_id = ?
                ORDER BY sort_order ASC, created_at ASC
                """,
                (rs, rowNum) -> toItemMap(rs),
                tenantId,
                propertyId);
    }

    @PortakiQueryHandler(value = "checklist.completions", scope = "stay:read")
    public List<String> listCompletions(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID stayId = UUID.fromString(ctx.stayId());
        return jdbc.query(
                """
                SELECT item_id::text
                FROM t_j_module_checklist_completions
                WHERE stay_id = ?
                """,
                (rs, rowNum) -> rs.getString(1),
                stayId);
    }

    @PortakiCommandHandler(value = "checklist.complete-item", scope = "checklist:write")
    public void completeItem(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID stayId = UUID.fromString(ctx.stayId());
        UUID tenantId = ctx.tenantIdInternal();
        UUID itemId = UUID.fromString(String.valueOf(params.get("itemId")));
        assertStayAndItem(tenantId, stayId, itemId);
        Boolean exists =
                jdbc.queryForObject(
                        """
                        SELECT EXISTS(
                          SELECT 1 FROM t_j_module_checklist_completions
                          WHERE stay_id = ? AND item_id = ?
                        )
                        """,
                        Boolean.class,
                        stayId,
                        itemId);
        if (Boolean.TRUE.equals(exists)) {
            return;
        }
        jdbc.update(
                """
                INSERT INTO t_j_module_checklist_completions (stay_id, item_id, completed_at)
                VALUES (?, ?, ?)
                """,
                stayId,
                itemId,
                Instant.now());
        int pct = refreshProgress(stayId, tenantId);
        ctx.publish(new PortakiModuleEvent("checklist.progress-updated", Map.of("percentage", pct)));
        if (pct == 100) {
            ctx.publish(new PortakiModuleEvent("checklist.completed", Map.of("stayId", ctx.stayId())));
        }
    }

    @PortakiCommandHandler(value = "checklist.uncomplete-item", scope = "checklist:write")
    public void uncompleteItem(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID stayId = UUID.fromString(ctx.stayId());
        UUID tenantId = ctx.tenantIdInternal();
        UUID itemId = UUID.fromString(String.valueOf(params.get("itemId")));
        assertStayAndItem(tenantId, stayId, itemId);
        jdbc.update(
                """
                DELETE FROM t_j_module_checklist_completions
                WHERE stay_id = ? AND item_id = ?
                """,
                stayId,
                itemId);
        int pct = refreshProgress(stayId, tenantId);
        ctx.publish(new PortakiModuleEvent("checklist.progress-updated", Map.of("percentage", pct)));
    }

    private void assertStayAndItem(UUID tenantId, UUID stayId, UUID itemId) {
        UUID propertyId =
                jdbc.queryForObject(
                        """
                        SELECT property_id FROM t_e_stays
                        WHERE id = ? AND tenant_id = ?
                        """,
                        UUID.class,
                        stayId,
                        tenantId);
        if (propertyId == null) {
            throw new IllegalArgumentException("stay_not_found");
        }
        Boolean itemOk =
                jdbc.queryForObject(
                        """
                        SELECT EXISTS(
                          SELECT 1 FROM t_e_module_checklist_items
                          WHERE id = ? AND tenant_id = ? AND property_id = ?
                        )
                        """,
                        Boolean.class,
                        itemId,
                        tenantId,
                        propertyId);
        if (!Boolean.TRUE.equals(itemOk)) {
            throw new IllegalArgumentException("item_not_found");
        }
    }

    private int refreshProgress(UUID stayId, UUID tenantId) {
        UUID propertyId =
                jdbc.queryForObject(
                        "SELECT property_id FROM t_e_stays WHERE id = ? AND tenant_id = ?",
                        UUID.class,
                        stayId,
                        tenantId);
        Long total =
                jdbc.queryForObject(
                        """
                        SELECT COUNT(*) FROM t_e_module_checklist_items
                        WHERE tenant_id = ? AND property_id = ?
                        """,
                        Long.class,
                        tenantId,
                        propertyId);
        Long done =
                jdbc.queryForObject(
                        "SELECT COUNT(*) FROM t_j_module_checklist_completions WHERE stay_id = ?",
                        Long.class,
                        stayId);
        long totalCount = total == null ? 0L : total;
        long doneCount = done == null ? 0L : done;
        int pct = totalCount == 0 ? 0 : (int) (doneCount * 100 / totalCount);
        jdbc.update(
                "UPDATE t_e_stays SET checklist_progress = ? WHERE id = ? AND tenant_id = ?",
                pct,
                stayId,
                tenantId);
        return pct;
    }

    private static Map<String, Object> toItemMap(ResultSet rs) throws SQLException {
        Map<String, Object> row = new HashMap<>();
        row.put("id", rs.getObject("id", UUID.class).toString());
        row.put("labelFr", rs.getString("label_fr"));
        row.put("labelEn", rs.getString("label_en"));
        row.put("sortOrder", rs.getInt("sort_order"));
        return row;
    }
}
