-- lost-found @ schema v3 — add the tenant `property_id` column.
--
-- v1 created `lost_found_report` without `property_id`, but the host typed-repo always writes it
-- (tenant isolation + row-level security). Every repo_create therefore failed with
-- "column property_id of relation lost_found_report does not exist". The table has stayed empty
-- (no create ever succeeded), so NOT NULL is safe.

ALTER TABLE module_lost_found.lost_found_report
    ADD COLUMN IF NOT EXISTS property_id UUID NOT NULL;

CREATE INDEX IF NOT EXISTS lost_found_report_property_idx
    ON module_lost_found.lost_found_report (property_id);
