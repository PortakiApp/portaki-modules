package app.portaki.module.events;

import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan(basePackageClasses = EventsGatewayModule.class)
public class EventsModuleBackendConfiguration {}
