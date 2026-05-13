package app.portaki.module.icalsync;

import java.time.Instant;
import java.util.List;
import java.util.Optional;

import com.fasterxml.jackson.databind.JsonNode;

import app.portaki.module.icalsync.calendar.IcalProviderType;
import app.portaki.module.icalsync.calendar.ParsedCalendarEvent;
import app.portaki.module.icalsync.internal.IcalDtParse;
import app.portaki.module.icalsync.parse.IcalCalendarParsers;

/**
 * Point d’entrée public du module : détection du fournisseur à partir d’un objet « feed » JSON, extraction des
 * VEVENT normalisés, et interprétation des propriétés de date/heure iCal (DTSTART / DTEND).
 */
public final class IcalFeedEventExtractor {

    private IcalFeedEventExtractor() {}

    public static IcalProviderType providerFromFeed(JsonNode feed) {
        if (feed == null || !feed.isObject()) {
            return IcalProviderType.GENERIC;
        }
        String explicit = feed.path("provider").asText("").trim().toUpperCase().replace('-', '_');
        if ("AIRBNB".equals(explicit)) {
            return IcalProviderType.AIRBNB;
        }
        if ("GENERIC".equals(explicit)) {
            return IcalProviderType.GENERIC;
        }
        String id = feed.path("id").asText("").toLowerCase();
        if (id.contains("airbnb")) {
            return IcalProviderType.AIRBNB;
        }
        String url = feed.path("url").asText("").toLowerCase();
        if (url.contains("airbnb.com") || url.contains("airbnb.fr")) {
            return IcalProviderType.AIRBNB;
        }
        return IcalProviderType.GENERIC;
    }

    public static List<ParsedCalendarEvent> parseBody(String rawIcsText, IcalProviderType providerType) {
        return IcalCalendarParsers.defaultResolver().resolve(providerType).parse(rawIcsText);
    }

    public static Optional<Instant> parseDateTimeProperty(String rawIcalValue) {
        return IcalDtParse.parseInstant(rawIcalValue);
    }
}
