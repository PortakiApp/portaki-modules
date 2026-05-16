package app.portaki.module.appliances;

import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan(basePackageClasses = AppliancesGatewayModule.class)
public class AppliancesModuleBackendConfiguration {}
