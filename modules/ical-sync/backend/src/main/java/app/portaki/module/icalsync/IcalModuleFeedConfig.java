package app.portaki.module.icalsync;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

/**
 * Résout la liste des flux iCal à partir des champs URL explicites, avec repli sur l’ancien champ
 * {@code feeds_json} (tableau JSON en texte).
 */
public final class IcalModuleFeedConfig {

    private IcalModuleFeedConfig() {}

    public static JsonNode resolveFeedsNode(ObjectMapper mapper, ObjectNode obj) throws JsonProcessingException {
        ArrayNode arr = mapper.createArrayNode();
        String primary = text(obj, "ical_url_primary");
        if (!primary.isBlank()) {
            ObjectNode f = mapper.createObjectNode();
            f.put("id", "primary");
            f.put("url", primary.trim());
            arr.add(f);
        }
        String secondary = text(obj, "ical_url_secondary");
        if (!secondary.isBlank()) {
            ObjectNode f = mapper.createObjectNode();
            f.put("id", "secondary");
            f.put("url", secondary.trim());
            arr.add(f);
        }
        if (!arr.isEmpty()) {
            return arr;
        }
        JsonNode legacy = obj.get("feeds_json");
        if (legacy != null && legacy.isTextual() && !legacy.asText().isBlank()) {
            return mapper.readTree(legacy.asText().trim());
        }
        return arr;
    }

    public static boolean hasExplicitIcalUrls(ObjectNode obj) {
        return !text(obj, "ical_url_primary").isBlank() || !text(obj, "ical_url_secondary").isBlank();
    }

    private static String text(ObjectNode obj, String key) {
        JsonNode n = obj.get(key);
        return n != null && n.isTextual() ? n.asText() : "";
    }
}
