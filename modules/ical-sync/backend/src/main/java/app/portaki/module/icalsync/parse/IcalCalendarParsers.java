package app.portaki.module.icalsync.parse;

import java.util.List;

import app.portaki.module.icalsync.calendar.IcalCalendarParserResolver;

/**
 * Fabrique des résolveurs de parseur iCal pour le module.
 */
public final class IcalCalendarParsers {

    private static final IcalCalendarParserResolver DEFAULT =
            new IcalCalendarParserResolver(List.of(new AirbnbIcalCalendarParserAdapter(), new GenericIcalCalendarParserAdapter()));

    private IcalCalendarParsers() {}

    public static IcalCalendarParserResolver defaultResolver() {
        return DEFAULT;
    }
}
