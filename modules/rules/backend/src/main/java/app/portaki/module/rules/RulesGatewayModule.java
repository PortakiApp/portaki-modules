package app.portaki.module.rules;

import java.sql.ResultSet;
import java.sql.SQLException;
import java.time.Instant;
import java.util.HashMap;
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
@PortakiModule("rules")
public class RulesGatewayModule {

    private final JdbcTemplate jdbc;
    private final ObjectMapper objectMapper;

    public RulesGatewayModule(JdbcTemplate jdbc, ObjectMapper objectMapper) {
        this.jdbc = jdbc;
        this.objectMapper = objectMapper;
    }

    @PortakiQueryHandler(value = "rules.content", scope = "property:read")
    public Map<String, Object> loadContent(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID tenantId = ctx.tenantIdInternal();
        UUID propertyId = UUID.fromString(ctx.propertyId());
        Map<String, Object> row =
                jdbc.query(
                        """
                        SELECT content_fr, content_en
                        FROM t_e_module_rules_content
                        WHERE tenant_id = ? AND property_id = ?
                        """,
                        rs -> {
                            if (!rs.next()) {
                                return Map.<String, Object>of();
                            }
                            return rowToContent(rs);
                        },
                        tenantId,
                        propertyId);
        return row == null ? Map.of() : row;
    }

    @PortakiCommandHandler(value = "rules.content.save", scope = "host:property:write")
    public void saveContent(Map<String, Object> params, GatewayModuleContext ctx) {
        UUID tenantId = ctx.tenantIdInternal();
        UUID propertyId = UUID.fromString(ctx.propertyId());
        JsonNode contentFr = readJsonParam(params, "contentFr");
        JsonNode contentEn = readJsonParam(params, "contentEn");
        Instant now = Instant.now();

        Boolean exists =
                jdbc.queryForObject(
                        """
                        SELECT EXISTS(
                          SELECT 1 FROM t_e_module_rules_content
                          WHERE tenant_id = ? AND property_id = ?
                        )
                        """,
                        Boolean.class,
                        tenantId,
                        propertyId);

        if (Boolean.TRUE.equals(exists)) {
            jdbc.update(
                    """
                    UPDATE t_e_module_rules_content
                    SET content_fr = ?::jsonb, content_en = ?::jsonb, updated_at = ?
                    WHERE tenant_id = ? AND property_id = ?
                    """,
                    jsonOrNull(contentFr),
                    jsonOrNull(contentEn),
                    now,
                    tenantId,
                    propertyId);
        } else {
            jdbc.update(
                    """
                    INSERT INTO t_e_module_rules_content (
                      id, tenant_id, property_id, content_fr, content_en, updated_at
                    ) VALUES (?, ?, ?, ?::jsonb, ?::jsonb, ?)
                    """,
                    UUID.randomUUID(),
                    tenantId,
                    propertyId,
                    jsonOrNull(contentFr),
                    jsonOrNull(contentEn),
                    now);
        }
    }

    private Map<String, Object> rowToContent(ResultSet rs) throws SQLException {
        Map<String, Object> m = new HashMap<>();
        putJson(m, "contentFr", rs.getString("content_fr"));
        putJson(m, "contentEn", rs.getString("content_en"));
        return m;
    }

    private void putJson(Map<String, Object> m, String key, String raw) throws SQLException {
        if (raw == null || raw.isBlank()) {
            return;
        }
        try {
            m.put(key, objectMapper.readValue(raw, Object.class));
        } catch (JsonProcessingException ignored) {
            /* skip */
        }
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
