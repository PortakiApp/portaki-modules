package dev.cyrilcolinet.portaki.module.trmnl;

import java.time.Instant;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;

import dev.cyrilcolinet.portaki.module.trmnl.model.DisplayMode;
import dev.cyrilcolinet.portaki.module.trmnl.model.TrmnlStayData;

import app.portaki.sdk.gateway.GatewayModuleContext;

/**
 * Construit le JSON {@code merge_variables} pour les templates Liquid TRMNL. Données compactes
 * (&lt; 2kb). Fusionne {@code trmnl_screen} (config hôte, optionnel) puis les champs de
 * l'événement Portaki.
 */
public final class TrmnlPayloadBuilder {

    private static final ZoneId DEFAULT_ZONE = ZoneId.of("Europe/Paris");
    private static final DateTimeFormatter TIME_FMT = DateTimeFormatter.ofPattern("HH:mm").withLocale(Locale.FRENCH);
    private static final DateTimeFormatter DATE_FMT =
            DateTimeFormatter.ofPattern("dd MMM yyyy").withLocale(Locale.FRENCH);

    public Map<String, Object> build(GatewayModuleContext ctx, DisplayMode mode, Map<String, Object> eventData) {
        return switch (mode) {
            case GUEST_DISPLAY -> buildGuestPayload(ctx, eventData);
            case HOST_DASHBOARD -> buildHostDashboardPayload(ctx, eventData);
        };
    }

    private Map<String, Object> buildHostDashboardPayload(GatewayModuleContext ctx, Map<String, Object> eventData) {
        Map<String, Object> layer = new LinkedHashMap<>();
        copyScreenMap(ctx, layer);
        if (eventData != null) {
            layer.putAll(eventData);
        }

        Map<String, Object> payload = new LinkedHashMap<>();
        payload.put("property_name", propertyName(ctx, layer));
        payload.put("updated_at", TIME_FMT.format(Instant.now().atZone(DEFAULT_ZONE)));
        payload.put("display_mode", "host_dashboard");

        payload.put("current_stay_guest", layer.get("current_stay_guest"));
        payload.put("current_stay_checkout", layer.get("current_stay_checkout"));
        putInt(payload, "checklist_progress", layer.get("checklist_progress"), 0);
        payload.put("next_stay_guest", layer.get("next_stay_guest"));
        payload.put("next_stay_checkin", layer.get("next_stay_checkin"));
        payload.put("next_stay_countdown_days", layer.get("next_stay_countdown_days"));
        putInt(payload, "alerts_count", layer.get("alerts_count"), 0);
        int alerts = ((Number) payload.get("alerts_count")).intValue();
        payload.put("has_alerts", alerts > 0);

        return payload;
    }

    private Map<String, Object> buildGuestPayload(GatewayModuleContext ctx, Map<String, Object> eventData) {
        Map<String, Object> layer = new LinkedHashMap<>();
        copyScreenMap(ctx, layer);
        if (eventData != null) {
            layer.putAll(eventData);
        }
        TrmnlStayData stay = TrmnlStayData.fromEventMap(layer);

        Map<String, Object> payload = new LinkedHashMap<>();
        payload.put("property_name", propertyName(ctx, layer));
        payload.put("display_mode", "guest_display");
        payload.put("updated_at", TIME_FMT.format(Instant.now().atZone(DEFAULT_ZONE)));

        String first = stay.guestFirstName().isBlank() ? "" : stay.guestFirstName();
        payload.put("guest_name", first.isBlank() ? "Bienvenue" : first);
        payload.put("checkin_date", nz(stay.checkinDate()));
        payload.put("checkout_date", nz(stay.checkoutDate()));
        payload.put("checkin_time", nz(stay.checkinTime()));
        payload.put("checkout_time", nz(stay.checkoutTime()));
        payload.put("wifi_ssid", nz(stay.wifiSsid()));
        payload.put("door_code", nz(stay.doorCode()));
        payload.put("stay_active", stay.stayActive());
        payload.put("next_local_event", Objects.toString(layer.get("next_local_event"), ""));

        return payload;
    }

    private static void copyScreenMap(GatewayModuleContext ctx, Map<String, Object> target) {
        Object raw = ctx.config().get("trmnl_screen");
        if (raw instanceof Map<?, ?> m) {
            for (Map.Entry<?, ?> e : m.entrySet()) {
                if (e.getKey() != null && e.getValue() != null) {
                    target.put(String.valueOf(e.getKey()), e.getValue());
                }
            }
        }
    }

    private static String propertyName(GatewayModuleContext ctx, Map<String, Object> layer) {
        String override = (String) ctx.config().get("property_name_override");
        if (override != null && !override.isBlank()) {
            return override.trim();
        }
        Object fromLayer = layer.get("property_name");
        if (fromLayer != null && !String.valueOf(fromLayer).isBlank()) {
            return String.valueOf(fromLayer).trim();
        }
        return "Portaki";
    }

    private static void putInt(Map<String, Object> payload, String key, Object raw, int defaultValue) {
        int v = defaultValue;
        if (raw instanceof Number n) {
            v = n.intValue();
        }
        payload.put(key, v);
    }

    private static String nz(String s) {
        return s == null ? "" : s;
    }

    public static String formatInstantDate(Instant instant, ZoneId zone) {
        if (instant == null) {
            return "";
        }
        return DATE_FMT.format(instant.atZone(zone == null ? DEFAULT_ZONE : zone));
    }
}
