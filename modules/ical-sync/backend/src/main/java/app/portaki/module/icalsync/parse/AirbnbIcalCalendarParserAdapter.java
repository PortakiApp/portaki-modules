package app.portaki.module.icalsync.parse;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

import app.portaki.module.icalsync.calendar.IcalCalendarParserPort;
import app.portaki.module.icalsync.calendar.IcalProviderType;
import app.portaki.module.icalsync.calendar.ParsedCalendarEvent;
import app.portaki.module.icalsync.internal.IcalTextParser;

/**
 * Parseur orienté flux Airbnb : même extraction VEVENT que le générique, avec normalisation minimale du corps
 * (BOM UTF-8). Exclut les blocs « disponibilité » sans réservation (ex. SUMMARY « Airbnb (Not available) »). Conserve
 * les séjours (ex. « Reserved » / « Réservé ») ainsi que les autres VEVENT non annulés.
 */
public final class AirbnbIcalCalendarParserAdapter implements IcalCalendarParserPort {

    @Override
    public boolean supports(IcalProviderType providerType) {
        return providerType == IcalProviderType.AIRBNB;
    }

    @Override
    public List<ParsedCalendarEvent> parse(String rawIcsText) {
        String body = stripUtf8Bom(rawIcsText == null ? "" : rawIcsText);
        List<ParsedCalendarEvent> out = new ArrayList<>();
        for (IcalTextParser.ParsedEvent e : IcalTextParser.parseEvents(body)) {
            ParsedCalendarEvent ev = GenericIcalCalendarParserAdapter.toDomain(e);
            if (ev.cancelled()) {
                continue;
            }
            if (isAirbnbAvailabilityNoise(ev)) {
                continue;
            }
            out.add(ev);
        }
        return out;
    }

    private static String stripUtf8Bom(String s) {
        if (s.startsWith("\uFEFF")) {
            return s.substring(1);
        }
        return s;
    }

    /**
     * Blocs calendrier Airbnb sans réservation associée (fermetures / indisponibilités), à ne pas traiter comme un
     * séjour importé.
     */
    private static boolean isAirbnbAvailabilityNoise(ParsedCalendarEvent ev) {
        String s = ev.summary();
        if (s == null) {
            return false;
        }
        String t = s.trim().toLowerCase(Locale.ROOT);
        return t.contains("not available") || t.contains("non disponible");
    }
}
