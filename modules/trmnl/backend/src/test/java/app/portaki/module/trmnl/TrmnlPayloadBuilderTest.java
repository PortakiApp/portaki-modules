package app.portaki.module.trmnl;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

import org.junit.jupiter.api.Test;

import app.portaki.module.trmnl.model.DisplayMode;

import app.portaki.sdk.gateway.GatewayModuleContext;
import app.portaki.sdk.gateway.ModuleManifestSnapshot;
import app.portaki.sdk.gateway.ScopeValidation;

class TrmnlPayloadBuilderTest {

    @Test
    void hostDashboardMergesTrmnlScreenAndEvent() {
        TrmnlPayloadBuilder builder = new TrmnlPayloadBuilder();
        GatewayModuleContext ctx = ctx(
                Map.of(
                        "display_mode",
                        "host_dashboard",
                        "trmnl_screen",
                        Map.of("next_stay_guest", "Screen"),
                        "property_name_override",
                        "Villa Test"));

        Map<String, Object> payload =
                builder.build(
                        ctx,
                        DisplayMode.HOST_DASHBOARD,
                        Map.of("next_stay_guest", "Event", "alerts_count", 2, "checklist_progress", 40));

        assertEquals("Villa Test", payload.get("property_name"));
        assertEquals("Event", payload.get("next_stay_guest"));
        assertEquals(40, ((Number) payload.get("checklist_progress")).intValue());
        assertEquals(2, ((Number) payload.get("alerts_count")).intValue());
        assertEquals(true, payload.get("has_alerts"));
        assertEquals("host_dashboard", payload.get("display_mode"));
    }

    @Test
    void guestDisplayUsesFirstNameOnly() {
        TrmnlPayloadBuilder builder = new TrmnlPayloadBuilder();
        GatewayModuleContext ctx = ctx(Map.of("display_mode", "guest_display"));
        Map<String, Object> payload =
                builder.build(
                        ctx,
                        DisplayMode.GUEST_DISPLAY,
                        Map.of(
                                "guest_first_name",
                                "Alex",
                                "wifi_ssid",
                                "Gite-Guest",
                                "door_code",
                                "1234",
                                "stay_active",
                                true));

        assertEquals("Alex", payload.get("guest_name"));
        assertEquals("Gite-Guest", payload.get("wifi_ssid"));
        assertEquals(true, payload.get("stay_active"));
    }

    @Test
    void guestDisplayDefaultWelcomeLabel() {
        TrmnlPayloadBuilder builder = new TrmnlPayloadBuilder();
        GatewayModuleContext ctx = ctx(Map.of("display_mode", "guest_display"));
        Map<String, Object> payload = builder.build(ctx, DisplayMode.GUEST_DISPLAY, Map.of());
        assertEquals("Bienvenue", payload.get("guest_name"));
    }

    @Test
    void rateLimiterAllowsBurstUnderCap() {
        TrmnlWebhookRateLimiter limiter = new TrmnlWebhookRateLimiter();
        String key = "https://example.com/hook";
        for (int i = 0; i < 11; i++) {
            assertTrue(limiter.tryAcquire(key), "iteration " + i);
        }
        assertTrue(!limiter.tryAcquire(key));
    }

    private static GatewayModuleContext ctx(Map<String, Object> config) {
        ModuleManifestSnapshot manifest =
                ModuleManifestSnapshot.of("trmnl", List.of("stay:read"), List.of(), List.of(), List.of());
        ScopeValidation scopes = new ScopeValidation(Map.of("trmnl", manifest));
        return new GatewayModuleContext(
                "trmnl",
                UUID.randomUUID().toString(),
                UUID.randomUUID().toString(),
                UUID.randomUUID(),
                Set.of("stay:read"),
                config,
                e -> {},
                scopes);
    }
}
