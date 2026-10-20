//! The Portaki conformance battery — what every module owes the platform.

mod sample;

use portaki_sdk::EmailTemplateKey;
use portaki_test_utils::MockContextBuilder;

fn email_fixture(_template: EmailTemplateKey, builder: MockContextBuilder) -> MockContextBuilder {
    sample::seeded(builder)
}

portaki_test_utils::conformance!(email_fixture = email_fixture);
