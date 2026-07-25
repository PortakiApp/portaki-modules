-- checklist @ schema v2 — typed-repo always scopes by property_id

ALTER TABLE module_checklist.checklist_completion
    ADD COLUMN IF NOT EXISTS property_id UUID;

-- Backfill from the related checklist item when possible.
UPDATE module_checklist.checklist_completion AS completion
SET property_id = item.property_id
FROM module_checklist.checklist_item AS item
WHERE completion.item_id = item.id
  AND completion.property_id IS NULL;

-- Orphan rows without a property scope cannot be read/written by typed-repo.
DELETE FROM module_checklist.checklist_completion
WHERE property_id IS NULL;

ALTER TABLE module_checklist.checklist_completion
    ALTER COLUMN property_id SET NOT NULL;

CREATE INDEX IF NOT EXISTS checklist_completion_property_idx
    ON module_checklist.checklist_completion (property_id);
