package app.portaki.module.events;

import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.jdbc.core.ResultSetExtractor;
import org.springframework.stereotype.Component;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import app.portaki.sdk.gateway.GatewayModuleContext;
import app.portaki.sdk.gateway.PortakiQueryHandler;
import app.portaki.sdk.module.PortakiModule;

@Component
@PortakiModule("events")
public class EventsGatewayModule {

    private final JdbcTemplate jdbc;
    private final ObjectMapper objectMapper;

    public EventsGatewayModule(JdbcTemplate jdbc, ObjectMapper objectMapper) {
        this.jdbc = jdbc;
        this.objectMapper = objectMapper;
    }

    @PortakiQueryHandler(value = "events.nearby", scope = "stay:read")
    public List<Map<String, Object>> listNearby(Map<String, Object> params, GatewayModuleContext ctx) {
        String cacheKey = "events|" + ctx.tenantIdInternal() + "|" + UUID.fromString(ctx.propertyId());
        return jdbc.query(
                """
                SELECT payload, expires_at
                FROM t_e_module_events_cache
                WHERE cache_key = ?
                """,
                (ResultSetExtractor<List<Map<String, Object>>>) this::readCachePayload,
                cacheKey);
    }

    private List<Map<String, Object>> readCachePayload(java.sql.ResultSet rs) throws java.sql.SQLException {
        if (!rs.next()) {
            return List.of();
        }
        Instant expiresAt = rs.getTimestamp("expires_at").toInstant();
        if (!expiresAt.isAfter(Instant.now())) {
            return List.of();
        }
        try {
            JsonNode payload = objectMapper.readTree(rs.getString("payload"));
            return payloadAsList(payload);
        } catch (com.fasterxml.jackson.core.JsonProcessingException e) {
            throw new IllegalStateException("events_cache_payload_invalid", e);
        }
    }

    private List<Map<String, Object>> payloadAsList(JsonNode payload) {
        if (payload == null || !payload.isArray()) {
            return List.of();
        }
        List<Map<String, Object>> out = new ArrayList<>();
        for (JsonNode node : payload) {
            out.add(objectMapper.convertValue(node, new TypeReference<Map<String, Object>>() {}));
        }
        return out;
    }
}
