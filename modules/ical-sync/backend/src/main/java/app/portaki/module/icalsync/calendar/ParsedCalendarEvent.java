package app.portaki.module.icalsync.calendar;

/**
 * Événement calendaire normalisé après parsing (dates encore brutes iCal, à interpréter côté hôte).
 */
public record ParsedCalendarEvent(
        String uid,
        String summary,
        String dtStartRaw,
        String dtEndRaw,
        String statusRaw) {

    public boolean cancelled() {
        if (statusRaw == null || statusRaw.isBlank()) {
            return false;
        }
        return "CANCELLED".equalsIgnoreCase(statusRaw.trim());
    }
}
