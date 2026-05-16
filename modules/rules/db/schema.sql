-- Canonical DDL for module `rules` — applied via portaki-api Flyway until module migrator (Atlas).

CREATE TABLE t_e_module_rules_content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL UNIQUE REFERENCES t_e_properties (id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES t_e_tenants (id),
    content_fr JSONB,
    content_en JSONB,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX t_e_module_rules_content_tenant_idx ON t_e_module_rules_content (tenant_id);
