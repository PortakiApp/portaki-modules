package app.portaki.module.icalsync.internal;

import java.time.Instant;
import java.time.LocalDate;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.time.format.DateTimeParseException;
import java.util.Optional;

public final class IcalDtParse {

    private static final DateTimeFormatter COMPACT_LOCAL =
            DateTimeFormatter.ofPattern("uuuuMMdd'T'HHmmss");

    private IcalDtParse() {}

    public static Optional<Instant> parseInstant(String raw) {
        if (raw == null || raw.isBlank()) {
            return Optional.empty();
        }
        String s = raw.trim();
        int semi = s.indexOf(';');
        if (semi > 0) {
            s = s.substring(semi + 1);
            int colon = s.indexOf(':');
            if (colon > 0 && colon < s.length() - 1) {
                s = s.substring(colon + 1).trim();
            }
        }
        if (s.length() == 8 && s.chars().allMatch(Character::isDigit)) {
            try {
                return Optional.of(
                        LocalDate.parse(s, DateTimeFormatter.BASIC_ISO_DATE)
                                .atStartOfDay(ZoneOffset.UTC)
                                .toInstant());
            } catch (DateTimeParseException e) {
                return Optional.empty();
            }
        }
        if (s.contains("T")) {
            if (s.endsWith("Z")) {
                try {
                    return Optional.of(Instant.parse(s));
                } catch (DateTimeParseException ignored) {
                    /* fall through */
                }
            }
            try {
                LocalDateTime ldt = LocalDateTime.parse(s, COMPACT_LOCAL);
                return Optional.of(ldt.atZone(ZoneOffset.UTC).toInstant());
            } catch (DateTimeParseException ignored) {
                return Optional.empty();
            }
        }
        return Optional.empty();
    }
}
