package dev.cyrilcolinet.portaki.module.trmnl;

import java.util.Map;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import com.fasterxml.jackson.databind.ObjectMapper;

import dev.cyrilcolinet.portaki.module.trmnl.model.DisplayMode;

import app.portaki.sdk.gateway.GatewayModuleContext;
import app.portaki.sdk.gateway.PortakiEventHandler;
import app.portaki.sdk.module.PortakiModule;

/**
 * Handlers d'événements module — à invoquer depuis l'hôte Portaki lorsque le bus interne relaie
 * les événements listés dans {@code portaki.module.json}.
 */
@PortakiModule("trmnl")
public final class TrmnlModule {

    private static final Logger log = LoggerFactory.getLogger(TrmnlModule.class);

    private final TrmnlWebhookClient webhookClient;
    private final TrmnlPayloadBuilder payloadBuilder;

    public TrmnlModule(TrmnlWebhookClient webhookClient, TrmnlPayloadBuilder payloadBuilder) {
        this.webhookClient = webhookClient;
        this.payloadBuilder = payloadBuilder;
    }

    public static TrmnlModule createDefault() {
        ObjectMapper mapper = new ObjectMapper();
        return new TrmnlModule(new TrmnlWebhookClient(mapper), new TrmnlPayloadBuilder());
    }

    @PortakiEventHandler("stay.status-changed")
    public void onStayStatusChanged(Map<String, Object> event, GatewayModuleContext ctx) {
        pushFullReplace(ctx, event);
    }

    @PortakiEventHandler("stay.created")
    public void onStayCreated(Map<String, Object> event, GatewayModuleContext ctx) {
        pushFullReplace(ctx, event);
    }

    @PortakiEventHandler("stay.deleted")
    public void onStayDeleted(Map<String, Object> event, GatewayModuleContext ctx) {
        pushFullReplace(ctx, event);
    }

    @PortakiEventHandler("checklist.progress-updated")
    public void onChecklistProgress(Map<String, Object> event, GatewayModuleContext ctx) {
        String webhookUrl = getWebhookUrl(ctx);
        if (webhookUrl == null) {
            return;
        }
        if (DisplayMode.fromConfig(ctx.config().get("display_mode")) != DisplayMode.HOST_DASHBOARD) {
            return;
        }
        int progress = ((Number) event.getOrDefault("percentage", 0)).intValue();
        Map<String, Object> patch =
                Map.of(
                        "checklist_progress", progress,
                        "checklist_completed", progress == 100);
        webhookClient.push(webhookUrl, patch, TrmnlWebhookClient.Strategy.DEEP_MERGE);
        log.debug("[portaki-trmnl] Checklist progress pushed: {}% for stay {}", progress, ctx.stayId());
    }

    @PortakiEventHandler("checklist.completed")
    public void onChecklistCompleted(Map<String, Object> event, GatewayModuleContext ctx) {
        pushFullReplace(ctx, event);
    }

    private void pushFullReplace(GatewayModuleContext ctx, Map<String, Object> event) {
        String webhookUrl = getWebhookUrl(ctx);
        if (webhookUrl == null) {
            return;
        }
        DisplayMode mode = DisplayMode.fromConfig(ctx.config().get("display_mode"));
        Map<String, Object> payload = payloadBuilder.build(ctx, mode, event);
        webhookClient.push(webhookUrl, payload, TrmnlWebhookClient.Strategy.REPLACE);
        log.info(
                "[portaki-trmnl] Pushed {} update for property {}",
                ctx.config().getOrDefault("display_mode", "host_dashboard"),
                ctx.propertyId());
    }

    private String getWebhookUrl(GatewayModuleContext ctx) {
        Object raw = ctx.config().get("webhook_url");
        String url = raw == null ? null : String.valueOf(raw).trim();
        if (url == null || url.isBlank()) {
            log.warn("[portaki-trmnl] No webhook URL configured for property {}", ctx.propertyId());
            return null;
        }
        return url;
    }
}
