-- consumables @ schema v3 — typed-repo scopes every table by property_id

ALTER TABLE module_consumables.consumable_report
    ADD COLUMN IF NOT EXISTS property_id UUID;

-- Backfill from the reported catalog item.
UPDATE module_consumables.consumable_report AS report
SET property_id = item.property_id
FROM module_consumables.consumable_item AS item
WHERE report.item_id = item.id
  AND report.property_id IS NULL;

-- Reports whose item is gone have no property scope; typed-repo can never reach them.
DELETE FROM module_consumables.consumable_report
WHERE property_id IS NULL;

ALTER TABLE module_consumables.consumable_report
    ALTER COLUMN property_id SET NOT NULL;

-- listRecent / listOpenCount / stats filter on property_id; listForStay adds stay_id.
CREATE INDEX IF NOT EXISTS consumable_report_property_stay_idx
    ON module_consumables.consumable_report (property_id, stay_id);
