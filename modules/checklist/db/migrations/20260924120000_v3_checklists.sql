-- checklist @ schema v3 — several lists per property (guest or host), host task state.
--
-- Existing items become the guest list « Checklist de départ » of their property; guest
-- completions keep pointing at the same item ids.

CREATE TABLE IF NOT EXISTS module_checklist.checklist (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL,
    name_fr TEXT NOT NULL DEFAULT '',
    name_en TEXT NOT NULL DEFAULT '',
    audience TEXT NOT NULL DEFAULT 'guest',
    icon TEXT NOT NULL DEFAULT 'check-circle',
    trigger TEXT NOT NULL DEFAULT 'duringStay',
    placement TEXT NOT NULL DEFAULT 'booklet',
    assignee_name TEXT,
    assignee_role TEXT,
    deadline TEXT,
    notify_assignee BOOLEAN NOT NULL DEFAULT false,
    alert_host BOOLEAN NOT NULL DEFAULT false,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS checklist_property_sort_idx
    ON module_checklist.checklist (property_id, sort_order);

ALTER TABLE module_checklist.checklist_item
    ADD COLUMN IF NOT EXISTS checklist_id UUID;
ALTER TABLE module_checklist.checklist_item
    ADD COLUMN IF NOT EXISTS photo_required BOOLEAN NOT NULL DEFAULT false;

-- One guest list per property that already has items. `duringStay` is the former default
-- (`from_checkin`); a host who picked another moment gets it back from the KV config on the
-- first read (see `storage::adopt_legacy_config`).
INSERT INTO module_checklist.checklist (id, property_id, name_fr, name_en, audience, icon, trigger, placement)
SELECT gen_random_uuid(), orphans.property_id, 'Checklist de départ', 'Checkout checklist',
       'guest', 'logout', 'duringStay', 'booklet'
FROM (
    SELECT DISTINCT property_id
    FROM module_checklist.checklist_item
    WHERE checklist_id IS NULL
) AS orphans
WHERE NOT EXISTS (
    SELECT 1 FROM module_checklist.checklist AS existing
    WHERE existing.property_id = orphans.property_id AND existing.audience = 'guest'
);

UPDATE module_checklist.checklist_item AS item
SET checklist_id = list.id
FROM module_checklist.checklist AS list
WHERE item.checklist_id IS NULL
  AND list.property_id = item.property_id
  AND list.audience = 'guest';

ALTER TABLE module_checklist.checklist_item
    ALTER COLUMN checklist_id SET NOT NULL;

CREATE INDEX IF NOT EXISTS checklist_item_list_idx
    ON module_checklist.checklist_item (checklist_id, sort_order);

CREATE TABLE IF NOT EXISTS module_checklist.task_item_state (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL,
    task_id TEXT NOT NULL,
    item_id UUID NOT NULL,
    done BOOLEAN NOT NULL DEFAULT false,
    photo TEXT,
    done_at TIMESTAMPTZ,
    CONSTRAINT task_item_state_task_item_uq UNIQUE (property_id, task_id, item_id)
);

CREATE INDEX IF NOT EXISTS task_item_state_task_idx
    ON module_checklist.task_item_state (property_id, task_id);
