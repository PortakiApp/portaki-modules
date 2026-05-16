package app.portaki.module.prearrivalform;

import java.util.Optional;

import app.portaki.sdk.email.LocalizedText;
import app.portaki.sdk.email.ModuleEmailContent;
import app.portaki.sdk.email.ModuleEmailCta;
import app.portaki.sdk.email.ModuleGuestEmailAction;
import app.portaki.sdk.email.PortakiModuleEmail;
import app.portaki.sdk.email.StayEmailContext;
import app.portaki.sdk.event.StayCreatedEvent;
import app.portaki.sdk.module.ModuleContext;
import app.portaki.sdk.module.OnEvent;
import app.portaki.sdk.module.PortakiModule;
import app.portaki.sdk.module.WebhookResponse;

@PortakiModule("pre-arrival-form")
public class PreArrivalFormModule {

    @OnEvent("StayCreatedEvent")
    public WebhookResponse onStayCreated(StayCreatedEvent event, ModuleContext ctx) {
        return WebhookResponse.ok();
    }

    @PortakiModuleEmail("reminder-day-before-arrival")
    public Optional<ModuleEmailContent> reminderDayBeforeArrival(StayEmailContext ctx) {
        if (ctx.guestEmail().isEmpty()) {
            return Optional.empty();
        }
        return Optional.of(
                ModuleEmailContent.of(
                        LocalizedText.of(
                                "Demain : préparez votre arrivée",
                                "Tomorrow: get ready for your arrival"),
                        LocalizedText.of(
                                "Bonjour,\n\n"
                                        + "Votre séjour approche. Pour que votre hôte puisse vous accueillir sereinement, "
                                        + "merci de remplir le court formulaire de pré-arrivée (horaire, allergies, etc.).\n\n"
                                        + "Cela ne prend que quelques minutes.",
                                "Hello,\n\n"
                                        + "Your stay is almost here. To help your host welcome you smoothly, "
                                        + "please complete the short pre-arrival form (timing, allergies, etc.).\n\n"
                                        + "It only takes a few minutes."),
                        ModuleEmailCta.of(
                                LocalizedText.of("Remplir le formulaire", "Fill in the form"),
                                ModuleGuestEmailAction.openModule("pre-arrival-form", "fill-form"))));
    }
}
