package app.portaki.module.icalsync;

import java.time.Instant;
import java.util.List;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;

import app.portaki.module.icalsync.calendar.IcalProviderType;
import app.portaki.module.icalsync.calendar.ParsedCalendarEvent;
import app.portaki.sdk.module.backend.HostModuleAction;
import app.portaki.sdk.module.backend.HostModuleRunResult;
import app.portaki.sdk.module.backend.ModuleBackendException;
import app.portaki.sdk.module.backend.ModuleHostContext;
import app.portaki.sdk.module.backend.PortakiHostModuleBackend;
import app.portaki.sdk.module.backend.http.SafeHttpsUtf8Fetcher;

/**
 * Backend hôte iCal : fetch des flux HTTPS, parsing VEVENT, mise à jour du résumé dans la config module.
 */
public class IcalSyncHostModuleBackend implements PortakiHostModuleBackend {

    private final ObjectMapper objectMapper;

    public IcalSyncHostModuleBackend(ObjectMapper objectMapper) {
        this.objectMapper = objectMapper;
    }

    @Override
    public String moduleId() {
        return "ical-sync";
    }

    @Override
    public HostModuleRunResult run(ModuleHostContext ctx, HostModuleAction action, String plainConfigJson)
            throws ModuleBackendException {
        if (!action.equals(HostModuleAction.SYNC)) {
            throw new ModuleBackendException("unsupported_action", action.value());
        }
        JsonNode root;
        try {
            root = objectMapper.readTree(plainConfigJson);
        } catch (JsonProcessingException e) {
            throw new ModuleBackendException("ical_sync_config_parse_failed", e.getMessage(), e);
        }
        if (!(root instanceof ObjectNode obj)) {
            throw new ModuleBackendException("ical_sync_config_invalid", "root must be object");
        }
        JsonNode feedsJsonNode = obj.get("feeds_json");
        if (feedsJsonNode == null || !feedsJsonNode.isTextual() || feedsJsonNode.asText().isBlank()) {
            throw new ModuleBackendException("feeds_json_required", "feeds_json required");
        }
        String feedsJson = feedsJsonNode.asText().trim();
        JsonNode feedsNode;
        try {
            feedsNode = objectMapper.readTree(feedsJson);
        } catch (JsonProcessingException e) {
            throw new ModuleBackendException("feeds_json_invalid", e.getMessage(), e);
        }
        if (!feedsNode.isArray()) {
            throw new ModuleBackendException("feeds_json_must_array", "feeds_json must be array");
        }
        if (feedsNode.isEmpty()) {
            throw new ModuleBackendException("feeds_json_empty", "feeds_json empty");
        }
        int okFeeds = 0;
        int failFeeds = 0;
        int events = 0;
        StringBuilder summary = new StringBuilder();
        for (JsonNode feed : feedsNode) {
            if (!feed.isObject()) {
                failFeeds++;
                summary.append("[invalid feed object]\n");
                continue;
            }
            String feedId = feed.path("id").asText("feed");
            String url = feed.path("url").asText("").trim();
            if (url.isEmpty()) {
                failFeeds++;
                summary.append("[").append(feedId).append("] empty url\n");
                continue;
            }
            try {
                String body = SafeHttpsUtf8Fetcher.fetch(url);
                IcalProviderType provider = IcalFeedEventExtractor.providerFromFeed(feed);
                List<ParsedCalendarEvent> ev = IcalFeedEventExtractor.parseBody(body, provider);
                events += ev.size();
                okFeeds++;
                summary.append("[")
                        .append(feedId)
                        .append("] ok — ")
                        .append(ev.size())
                        .append(" events\n");
            } catch (ModuleBackendException e) {
                failFeeds++;
                summary.append("[")
                        .append(feedId)
                        .append("] error: ")
                        .append(e.code())
                        .append("\n");
            } catch (Exception e) {
                failFeeds++;
                summary.append("[")
                        .append(feedId)
                        .append("] error: ")
                        .append(e.getClass().getSimpleName())
                        .append("\n");
            }
        }
        ObjectNode merged = obj.deepCopy();
        merged.put("last_sync_at", Instant.now().toString());
        merged.put("sync_summary", summary.toString().trim());
        String updatedPlain;
        try {
            updatedPlain = objectMapper.writeValueAsString(merged);
        } catch (JsonProcessingException e) {
            throw new ModuleBackendException("ical_sync_serialize_failed", e.getMessage(), e);
        }
        return new HostModuleRunResult(true, okFeeds, failFeeds, events, summary.toString().trim(), updatedPlain);
    }
}
