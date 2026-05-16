-- Module appliances @ v1.0.0 (idempotent)

CREATE TABLE IF NOT EXISTS t_e_module_appliances_content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    property_id UUID NOT NULL UNIQUE REFERENCES t_e_properties (id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES t_e_tenants (id),
    content_fr JSONB,
    content_en JSONB,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS t_e_module_appliances_content_tenant_idx
    ON t_e_module_appliances_content (tenant_id);
