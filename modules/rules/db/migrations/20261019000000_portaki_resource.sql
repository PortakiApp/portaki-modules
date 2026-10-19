-- rules: the collections the SDK stores (#[collection]) — written by portaki build.

CREATE SCHEMA IF NOT EXISTS module_rules;

CREATE TABLE IF NOT EXISTS module_rules.portaki_resource (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL,
    resource TEXT NOT NULL,
    rows_json TEXT NOT NULL DEFAULT '[]',
    meta_json TEXT NOT NULL DEFAULT '{}',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS portaki_resource_property_resource_idx
    ON module_rules.portaki_resource (property_id, resource);
