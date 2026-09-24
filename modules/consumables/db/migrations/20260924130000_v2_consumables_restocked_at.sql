-- consumables @ schema v2 — when the host marked a report restocked (« Dernier réassort »).

ALTER TABLE module_consumables.consumable_report
    ADD COLUMN IF NOT EXISTS restocked_at TIMESTAMPTZ;
