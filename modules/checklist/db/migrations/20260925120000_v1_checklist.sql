-- checklist @ schema v1 — idempotent (module_checklist)
--
-- Several lists per property (guest or host), their items, guest completions and host task state.
-- `trigger` is quoted: the platform SQL policy refuses the bare keyword, whatever its role.

CREATE SCHEMA IF NOT EXISTS module_checklist;

CREATE TABLE IF NOT EXISTS module_checklist.checklist (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL,
    name_fr TEXT NOT NULL DEFAULT '',
    name_en TEXT NOT NULL DEFAULT '',
    audience TEXT NOT NULL DEFAULT 'guest',
    icon TEXT NOT NULL DEFAULT 'check-circle',
    "trigger" TEXT NOT NULL DEFAULT 'duringStay',
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

CREATE TABLE IF NOT EXISTS module_checklist.checklist_item (
    id UUID PRIMARY KEY,
    property_id UUID NOT NULL,
    label_fr TEXT NOT NULL DEFAULT '',
    label_en TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    checklist_id UUID NOT NULL,
    photo_required BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS checklist_item_property_sort_idx
    ON module_checklist.checklist_item (property_id, sort_order);

CREATE INDEX IF NOT EXISTS checklist_item_list_idx
    ON module_checklist.checklist_item (checklist_id, sort_order);

CREATE TABLE IF NOT EXISTS module_checklist.checklist_completion (
    id UUID PRIMARY KEY,
    stay_id UUID NOT NULL,
    item_id UUID NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    property_id UUID NOT NULL,
    CONSTRAINT checklist_completion_stay_item_uq UNIQUE (stay_id, item_id)
);

CREATE INDEX IF NOT EXISTS checklist_completion_stay_idx
    ON module_checklist.checklist_completion (stay_id);

CREATE INDEX IF NOT EXISTS checklist_completion_property_idx
    ON module_checklist.checklist_completion (property_id);

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
