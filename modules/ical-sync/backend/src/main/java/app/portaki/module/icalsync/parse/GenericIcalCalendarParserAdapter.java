package app.portaki.module.icalsync.parse;

import java.util.ArrayList;
import java.util.List;

import app.portaki.module.icalsync.calendar.IcalCalendarParserPort;
import app.portaki.module.icalsync.calendar.IcalProviderType;
import app.portaki.module.icalsync.calendar.ParsedCalendarEvent;
import app.portaki.module.icalsync.internal.IcalTextParser;

/**
 * Parseur iCal générique (RFC 5545 basique) : extraction VEVENT via {@link IcalTextParser}, exclusion des
 * événements {@code STATUS:CANCELLED}.
 */
public final class GenericIcalCalendarParserAdapter implements IcalCalendarParserPort {

    @Override
    public boolean supports(IcalProviderType providerType) {
        return providerType == IcalProviderType.GENERIC;
    }

    @Override
    public List<ParsedCalendarEvent> parse(String rawIcsText) {
        List<ParsedCalendarEvent> out = new ArrayList<>();
        for (IcalTextParser.ParsedEvent e : IcalTextParser.parseEvents(rawIcsText)) {
            ParsedCalendarEvent ev = toDomain(e);
            if (!ev.cancelled()) {
                out.add(ev);
            }
        }
        return out;
    }

    static ParsedCalendarEvent toDomain(IcalTextParser.ParsedEvent e) {
        return new ParsedCalendarEvent(e.uid(), e.summary(), e.dtStart(), e.dtEnd(), e.status());
    }
}
