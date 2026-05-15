package app.portaki.module.trmnl;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.util.Map;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import com.fasterxml.jackson.databind.ObjectMapper;

/**
 * Client HTTP minimal (sans Spring) vers l'API webhook TRMNL.
 */
public final class TrmnlWebhookClient {

    private static final Logger log = LoggerFactory.getLogger(TrmnlWebhookClient.class);

    private final ObjectMapper objectMapper;
    private final HttpClient httpClient;
    private final TrmnlWebhookRateLimiter rateLimiter;

    public TrmnlWebhookClient(ObjectMapper objectMapper) {
        this(objectMapper, new TrmnlWebhookRateLimiter());
    }

    public TrmnlWebhookClient(ObjectMapper objectMapper, TrmnlWebhookRateLimiter rateLimiter) {
        this.objectMapper = objectMapper;
        this.rateLimiter = rateLimiter;
        this.httpClient =
                HttpClient.newBuilder()
                        .connectTimeout(Duration.ofSeconds(8))
                        .version(HttpClient.Version.HTTP_1_1)
                        .build();
    }

    public enum Strategy {
        REPLACE,
        DEEP_MERGE,
        STREAM
    }

    public void push(String webhookUrl, Map<String, Object> payload, Strategy strategy) {
        if (webhookUrl == null || webhookUrl.isBlank()) {
            return;
        }
        if (!rateLimiter.tryAcquire(webhookUrl.trim())) {
            log.warn("[portaki-trmnl] Rate limit reached for webhook URL (skipping push)");
            return;
        }
        try {
            Map<String, Object> body = buildBody(payload, strategy);
            String json = objectMapper.writeValueAsString(body);
            HttpRequest request =
                    HttpRequest.newBuilder()
                            .uri(URI.create(webhookUrl.trim()))
                            .timeout(Duration.ofSeconds(15))
                            .header("Content-Type", "application/json")
                            .header("User-Agent", "PortakiTrmnlModule/1.0")
                            .POST(HttpRequest.BodyPublishers.ofString(json))
                            .build();
            HttpResponse<Void> response = httpClient.send(request, HttpResponse.BodyHandlers.discarding());
            int code = response.statusCode();
            if (code < 200 || code >= 300) {
                log.error("[portaki-trmnl] Webhook returned HTTP {} for URL {}", code, maskUrl(webhookUrl));
            }
        } catch (Exception e) {
            log.error("[portaki-trmnl] Webhook push failed for URL {}: {}", maskUrl(webhookUrl), e.getMessage());
        }
    }

    private static Map<String, Object> buildBody(Map<String, Object> payload, Strategy strategy) {
        return switch (strategy) {
            case REPLACE -> Map.of("merge_variables", payload);
            case DEEP_MERGE -> Map.of("merge_variables", payload, "strategy", "deep_merge");
            case STREAM -> Map.of("merge_variables", payload, "strategy", "stream");
        };
    }

    private static String maskUrl(String url) {
        if (url.length() <= 24) {
            return url;
        }
        return url.substring(0, 20) + "…";
    }
}
