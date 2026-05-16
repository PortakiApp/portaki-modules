package app.portaki.module.appliances;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Map;
import java.util.UUID;

import org.junit.jupiter.api.Test;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.jdbc.core.ResultSetExtractor;

import com.fasterxml.jackson.databind.ObjectMapper;

import app.portaki.sdk.gateway.GatewayModuleContext;
import app.portaki.sdk.test.GatewayModuleContextBuilder;
import app.portaki.sdk.test.ModuleGatewayTestInvoker;

class AppliancesGatewayModuleTest {

    @Test
    void loadContent_whenNoRow_returnsEmptyMap() {
        AppliancesGatewayModule module =
                new AppliancesGatewayModule(new StubJdbc(Map.of()), new ObjectMapper());
        GatewayModuleContext ctx = appliancesContext();

        @SuppressWarnings("unchecked")
        Map<String, Object> result =
                (Map<String, Object>)
                        ModuleGatewayTestInvoker.invokeQuery(
                                module, "appliances.content", Map.of(), ctx);

        assertTrue(result.isEmpty());
    }

    @Test
    void loadContent_whenRowPresent_returnsContent() {
        Map<String, Object> row =
                Map.of("contentFr", Map.of("type", "doc"), "contentEn", Map.of("type", "doc"));
        AppliancesGatewayModule module =
                new AppliancesGatewayModule(new StubJdbc(row), new ObjectMapper());
        GatewayModuleContext ctx = appliancesContext();

        @SuppressWarnings("unchecked")
        Map<String, Object> result =
                (Map<String, Object>)
                        ModuleGatewayTestInvoker.invokeQuery(
                                module, "appliances.content", Map.of(), ctx);

        assertEquals("doc", ((Map<?, ?>) result.get("contentFr")).get("type"));
    }

    private static GatewayModuleContext appliancesContext() {
        return new GatewayModuleContextBuilder()
                .moduleId("appliances")
                .propertyId("11111111-1111-4111-8111-111111111111")
                .scopes("property:read")
                .scopeValidation(
                        GatewayModuleContextBuilder.scopeValidationFor(
                                "appliances", java.util.List.of("property:read"), java.util.List.of()))
                .build();
    }

  private static final class StubJdbc extends JdbcTemplate {

        private final Map<String, Object> queryResult;

        StubJdbc(Map<String, Object> queryResult) {
            this.queryResult = queryResult;
        }

        @Override
        public <T> T query(String sql, ResultSetExtractor<T> rse, Object... args) {
            @SuppressWarnings("unchecked")
            T typed = (T) queryResult;
            return typed;
        }
    }
}
