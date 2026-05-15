package app.portaki.module.trmnl.model;

public enum DisplayMode {

    HOST_DASHBOARD,
    GUEST_DISPLAY;

    public static DisplayMode fromConfig(Object raw) {
        if ("guest_display".equals(raw)) {
            return GUEST_DISPLAY;
        }
        return HOST_DASHBOARD;
    }
}
