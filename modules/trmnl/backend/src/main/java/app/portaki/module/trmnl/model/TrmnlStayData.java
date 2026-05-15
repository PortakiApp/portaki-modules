package app.portaki.module.trmnl.model;

import java.util.Map;

/**
 * Snapshot minimal pour l'écran TRMNL (aucun tenantId, pas d'email complet, prénom seulement côté
 * voyageur).
 */
public record TrmnlStayData(
        String guestFirstName,
        String checkinDate,
        String checkoutDate,
        String checkinTime,
        String checkoutTime,
        String wifiSsid,
        String doorCode,
        boolean stayActive) {

    public static TrmnlStayData fromEventMap(Map<String, Object> m) {
        if (m == null || m.isEmpty()) {
            return new TrmnlStayData("", "", "", "", "", "", "", false);
        }
        return new TrmnlStayData(
                str(m.get("guest_first_name")),
                str(m.get("checkin_date")),
                str(m.get("checkout_date")),
                str(m.get("checkin_time")),
                str(m.get("checkout_time")),
                str(m.get("wifi_ssid")),
                str(m.get("door_code")),
                bool(m.get("stay_active")));
    }

    private static String str(Object o) {
        return o == null ? "" : String.valueOf(o);
    }

    private static boolean bool(Object o) {
        if (o instanceof Boolean b) {
            return b;
        }
        if (o instanceof Number n) {
            return n.intValue() != 0;
        }
        return false;
    }
}
