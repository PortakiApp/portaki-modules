package app.portaki.module.train;

import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan(basePackageClasses = TrainGatewayModule.class)
public class TrainModuleBackendConfiguration {}
