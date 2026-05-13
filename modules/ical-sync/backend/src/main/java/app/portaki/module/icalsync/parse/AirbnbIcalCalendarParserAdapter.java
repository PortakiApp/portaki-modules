package app.portaki.module.icalsync.parse;

import java.util.ArrayList;
import java.util.List;

import app.portaki.module.icalsync.calendar.IcalCalendarParserPort;
import app.portaki.module.icalsync.calendar.IcalProviderType;
import app.portaki.module.icalsync.calendar.ParsedCalendarEvent;
import app.portaki.module.icalsync.internal.IcalTextParser;

/**
 * Parseur orienté flux Airbnb : même extraction VEVENT que le générique, avec normalisation minimale du corps
 * (BOM UTF-8) et filtrage des séjours annulés.
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
            if (isAirbnbReservedBlock(ev)) {
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
     * Airbnb exporte les nuits fermées / indisponibles comme des VEVENT dont le SUMMARY vaut « Reserved »
     * (ou variante) — ce ne sont pas des réservations à importer.
     */
    private static boolean isAirbnbReservedBlock(ParsedCalendarEvent ev) {
        String s = ev.summary();
        if (s == null) {
            return false;
        }
        String t = s.trim();
        return "reserved".equalsIgnoreCase(t) || "réservé".equalsIgnoreCase(t);
    }
}
