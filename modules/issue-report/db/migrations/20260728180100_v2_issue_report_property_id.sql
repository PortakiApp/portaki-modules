-- issue-report @ schema v2 — add the tenant `property_id` column.
--
-- v1 created `issue_report` without `property_id`, but the host typed-repo always writes it
-- (tenant isolation + row-level security), so every repo_create failed with
-- "column property_id of relation issue_report does not exist". The table has stayed empty
-- (no create ever succeeded), so NOT NULL is safe.

ALTER TABLE module_issue_report.issue_report
    ADD COLUMN IF NOT EXISTS property_id UUID NOT NULL;

CREATE INDEX IF NOT EXISTS issue_report_property_idx
    ON module_issue_report.issue_report (property_id);
