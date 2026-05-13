package app.portaki.module.icalsync.calendar;

import java.util.List;
import java.util.Objects;

/**
 * Sélectionne l’adaptateur de parsing en fonction du {@link IcalProviderType}.
 */
public final class IcalCalendarParserResolver {

    private final List<IcalCalendarParserPort> parsers;

    public IcalCalendarParserResolver(List<IcalCalendarParserPort> parsers) {
        this.parsers = List.copyOf(Objects.requireNonNull(parsers));
    }

    public IcalCalendarParserPort resolve(IcalProviderType type) {
        IcalProviderType t = type == null ? IcalProviderType.GENERIC : type;
        return parsers.stream()
                .filter(p -> p.supports(t))
                .findFirst()
                .orElseThrow(() -> new IllegalStateException("ical_parser_missing_for_" + t));
    }
}
