package app.portaki.module.sections;

import java.sql.ResultSet;
import java.sql.SQLException;
import java.time.Instant;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.stereotype.Component;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import app.portaki.sdk.gateway.GatewayModuleContext;
import app.portaki.sdk.gateway.PortakiCommandHandler;
import app.portaki.sdk.gateway.PortakiQueryHandler;
import app.portaki.sdk.module.PortakiModule;

@Component
@PortakiModule("sections")
public class SectionsGatewayModule {

    private final JdbcTemplate jdbc;
    private final ObjectMapper objectMapper;

    public SectionsGatewayModule(JdbcTemplate jdbc, ObjectMapper objectMapper) {
        this.jdbc = jdbc;
        this.objectMapper = objectMapper;
    }

    @PortakiQueryHandler(value = "sections.list", scope = "property:read")
    public Map<String, Object> listSections(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID tenantId = ctx.tenantIdInternal();
        UUID propertyId = UUID.fromString(ctx.propertyId());
        List<Map<String, Object>> rows =
                jdbc.query(
                        """
                        SELECT id, sort_order, title_fr, title_en, content_fr, content_en, updated_at
                        FROM t_e_module_sections_item
                        WHERE tenant_id = ? AND property_id = ?
                        ORDER BY sort_order ASC, created_at ASC
                        """,
                        (rs, rowNum) -> toSectionRow(rs),
                        tenantId,
                        propertyId);
        return Map.of("sections", rows);
    }

    @PortakiCommandHandler(value = "sections.section.save", scope = "host:property:write")
    public void saveSection(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID tenantId = ctx.tenantIdInternal();
        UUID propertyId = UUID.fromString(ctx.propertyId());
        String idRaw = stringParam(params, "id");
        UUID id = idRaw == null || idRaw.isBlank() ? UUID.randomUUID() : UUID.fromString(idRaw);
        String titleFr = stringParam(params, "titleFr");
        String titleEn = stringParam(params, "titleEn");
        JsonNode contentFr = readJsonParam(params, "contentFr");
        JsonNode contentEn = readJsonParam(params, "contentEn");
        int sortOrder = intParam(params, "sortOrder", nextSortOrder(tenantId, propertyId, id));

        if (titleFr == null || titleFr.isBlank()) {
            throw new IllegalArgumentException("title_fr_required");
        }
        if (titleEn == null || titleEn.isBlank()) {
            titleEn = titleFr;
        }

        boolean exists =
                Boolean.TRUE.equals(
                        jdbc.queryForObject(
                                """
                                SELECT EXISTS(
                                  SELECT 1 FROM t_e_module_sections_item
                                  WHERE id = ? AND tenant_id = ? AND property_id = ?
                                )
                                """,
                                Boolean.class,
                                id,
                                tenantId,
                                propertyId));

        if (exists) {
            jdbc.update(
                    """
                    UPDATE t_e_module_sections_item
                    SET title_fr = ?, title_en = ?, content_fr = ?::jsonb, content_en = ?::jsonb,
                        sort_order = ?, updated_at = ?
                    WHERE id = ? AND tenant_id = ? AND property_id = ?
                    """,
                    titleFr,
                    titleEn,
                    jsonOrNull(contentFr),
                    jsonOrNull(contentEn),
                    sortOrder,
                    Instant.now(),
                    id,
                    tenantId,
                    propertyId);
        } else {
            jdbc.update(
                    """
                    INSERT INTO t_e_module_sections_item (
                      id, tenant_id, property_id, sort_order, title_fr, title_en,
                      content_fr, content_en, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?::jsonb, ?::jsonb, ?, ?)
                    """,
                    id,
                    tenantId,
                    propertyId,
                    sortOrder,
                    titleFr,
                    titleEn,
                    jsonOrNull(contentFr),
                    jsonOrNull(contentEn),
                    Instant.now(),
                    Instant.now());
        }

    }

    @PortakiCommandHandler(value = "sections.section.delete", scope = "host:property:write")
    public void deleteSection(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID tenantId = ctx.tenantIdInternal();
        UUID propertyId = UUID.fromString(ctx.propertyId());
        String idRaw = stringParam(params, "id");
        if (idRaw == null || idRaw.isBlank()) {
            throw new IllegalArgumentException("id_required");
        }
        UUID id = UUID.fromString(idRaw);
        int deleted =
                jdbc.update(
                        """
                        DELETE FROM t_e_module_sections_item
                        WHERE id = ? AND tenant_id = ? AND property_id = ?
                        """,
                        id,
                        tenantId,
                        propertyId);
        if (deleted == 0) {
            throw new IllegalArgumentException("section_not_found");
        }
    }

    @PortakiCommandHandler(value = "sections.reorder", scope = "host:property:write")
    public void reorder(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID tenantId = ctx.tenantIdInternal();
        UUID propertyId = UUID.fromString(ctx.propertyId());
        Object raw = params.get("orderedIds");
        if (!(raw instanceof List<?> list) || list.isEmpty()) {
            throw new IllegalArgumentException("ordered_ids_required");
        }
        int order = 0;
        for (Object item : list) {
            if (item == null) {
                continue;
            }
            UUID id = UUID.fromString(item.toString());
            jdbc.update(
                    """
                    UPDATE t_e_module_sections_item
                    SET sort_order = ?, updated_at = ?
                    WHERE id = ? AND tenant_id = ? AND property_id = ?
                    """,
                    order++,
                    Instant.now(),
                    id,
                    tenantId,
                    propertyId);
        }
    }

    private Map<String, Object> toSectionRow(ResultSet rs) throws SQLException {
        Map<String, Object> row = new HashMap<>();
        row.put("id", rs.getObject("id", UUID.class).toString());
        row.put("sortOrder", rs.getInt("sort_order"));
        row.put("titleFr", rs.getString("title_fr"));
        row.put("titleEn", rs.getString("title_en"));
        row.put("contentFr", parseJsonColumn(rs.getString("content_fr")));
        row.put("contentEn", parseJsonColumn(rs.getString("content_en")));
        Instant updatedAt = rs.getObject("updated_at", Instant.class);
        if (updatedAt != null) {
            row.put("updatedAt", updatedAt.toString());
        }
        return row;
    }

    private Object parseJsonColumn(String raw) {
        if (raw == null || raw.isBlank()) {
            return null;
        }
        try {
            return objectMapper.readValue(raw, Object.class);
        } catch (JsonProcessingException e) {
            return null;
        }
    }

    private int nextSortOrder(UUID tenantId, UUID propertyId, UUID exceptId) {
        Integer max =
                jdbc.queryForObject(
                        """
                        SELECT COALESCE(MAX(sort_order), -1)
                        FROM t_e_module_sections_item
                        WHERE tenant_id = ? AND property_id = ? AND id <> ?
                        """,
                        Integer.class,
                        tenantId,
                        propertyId,
                        exceptId);
        return max == null ? 0 : max + 1;
    }

    private static String stringParam(Map<String, Object> params, String key) {
        Object v = params.get(key);
        return v == null ? null : v.toString();
    }

    private static int intParam(Map<String, Object> params, String key, int defaultValue) {
        Object v = params.get(key);
        if (v == null) {
            return defaultValue;
        }
        if (v instanceof Number n) {
            return n.intValue();
        }
        return Integer.parseInt(v.toString());
    }

    private JsonNode readJsonParam(Map<String, Object> params, String key) {
        Object v = params.get(key);
        if (v == null) {
            return null;
        }
        return objectMapper.valueToTree(v);
    }

    private String jsonOrNull(JsonNode node) {
        if (node == null || node.isNull()) {
            return null;
        }
        try {
            return objectMapper.writeValueAsString(node);
        } catch (JsonProcessingException e) {
            throw new IllegalArgumentException("invalid_json", e);
        }
    }
}
