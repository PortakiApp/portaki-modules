package app.portaki.module.trmnl;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class TrmnlModuleBackendConfiguration {

    @Bean
    TrmnlModule trmnlModule() {
        return TrmnlModule.createDefault();
    }
}
