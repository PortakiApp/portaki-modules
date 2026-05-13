package app.portaki.module.icalsync.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public final class IcalTextParser {

    private static final int MAX_EVENTS = 500;

    private static final Pattern PROP =
            Pattern.compile("(?im)^([A-Z-]+)(?:[^:\n]*):([^\\r\\n]+)");

    private IcalTextParser() {}

    public record ParsedEvent(String uid, String summary, String dtStart, String dtEnd) {}

    public static List<ParsedEvent> parseEvents(String raw) {
        if (raw == null || raw.isBlank()) {
            return List.of();
        }
        String unfolded = unfold(raw);
        List<ParsedEvent> out = new ArrayList<>();
        int cursor = 0;
        while (out.size() < MAX_EVENTS) {
            int start = unfolded.indexOf("BEGIN:VEVENT", cursor);
            if (start < 0) {
                break;
            }
            int end = unfolded.indexOf("END:VEVENT", start);
            if (end < 0) {
                break;
            }
            String block = unfolded.substring(start, end);
            String uid = firstProp(block, "UID");
            String summary = firstProp(block, "SUMMARY");
            String dtStart = firstProp(block, "DTSTART");
            String dtEnd = firstProp(block, "DTEND");
            if (!dtStart.isEmpty() || !summary.isEmpty() || !uid.isEmpty()) {
                out.add(new ParsedEvent(uid, summary, dtStart, dtEnd));
            }
            cursor = end + "END:VEVENT".length();
        }
        return out;
    }

    private static String unfold(String raw) {
        String n = raw.replace("\r\n", "\n");
        return n.replaceAll("\n[ \t]", "");
    }

    private static String firstProp(String block, String name) {
        Matcher m = PROP.matcher(block);
        while (m.find()) {
            if (name.equalsIgnoreCase(m.group(1))) {
                return m.group(2).trim();
            }
        }
        return "";
    }
}
