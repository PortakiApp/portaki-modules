-- consumables @ schema v1 — idempotent (module_consumables)

CREATE SCHEMA IF NOT EXISTS module_consumables;

CREATE TABLE IF NOT EXISTS module_consumables.consumable_item (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL,
    label_fr TEXT NOT NULL DEFAULT '',
    label_en TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    low_threshold INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS consumable_item_property_sort_idx
    ON module_consumables.consumable_item (property_id, sort_order);

CREATE TABLE IF NOT EXISTS module_consumables.consumable_report (
    id UUID PRIMARY KEY,
    stay_id UUID NOT NULL,
    item_id UUID NOT NULL,
    item_label TEXT NOT NULL,
    level TEXT NOT NULL,
    note TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS consumable_report_stay_idx
    ON module_consumables.consumable_report (stay_id);

CREATE INDEX IF NOT EXISTS consumable_report_status_idx
    ON module_consumables.consumable_report (status);
