package app.portaki.module.rules;

import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan(basePackageClasses = RulesGatewayModule.class)
public class RulesModuleBackendConfiguration {}
