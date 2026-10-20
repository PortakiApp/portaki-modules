-- checklist: the collections and records the SDK stores — written by portaki build.

CREATE SCHEMA IF NOT EXISTS module_checklist;

CREATE TABLE IF NOT EXISTS module_checklist.portaki_resource (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL,
    resource TEXT NOT NULL,
    rows_json TEXT NOT NULL DEFAULT '[]',
    meta_json TEXT NOT NULL DEFAULT '{}',
    stay_id UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS portaki_resource_property_resource_idx
    ON module_checklist.portaki_resource (property_id, resource);

CREATE INDEX IF NOT EXISTS portaki_resource_stay_idx
    ON module_checklist.portaki_resource (stay_id);
