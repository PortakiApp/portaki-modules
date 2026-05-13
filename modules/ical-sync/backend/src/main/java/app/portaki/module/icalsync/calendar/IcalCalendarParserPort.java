package app.portaki.module.icalsync.calendar;

import java.util.List;

/**
 * Port sortant : parse un corps de calendrier (.ics) pour un fournisseur donné.
 */
public interface IcalCalendarParserPort {

    boolean supports(IcalProviderType providerType);

    List<ParsedCalendarEvent> parse(String rawIcsText);
}
