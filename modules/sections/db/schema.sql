-- Canonical DDL for module `sections` (review in portaki-modules).
-- Runtime: copied to portaki-api Flyway until Atlas module migrator exists.

CREATE TABLE t_e_module_sections_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES t_e_tenants (id),
    property_id UUID NOT NULL REFERENCES t_e_properties (id) ON DELETE CASCADE,
    sort_order INT NOT NULL DEFAULT 0,
    title_fr TEXT NOT NULL DEFAULT '',
    title_en TEXT NOT NULL DEFAULT '',
    content_fr JSONB,
    content_en JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX t_e_module_sections_item_property_idx
    ON t_e_module_sections_item (tenant_id, property_id);
