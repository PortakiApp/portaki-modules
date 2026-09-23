-- issue-report @ schema v3 — host resolution timestamp (NULL = still open).

ALTER TABLE module_issue_report.issue_report
    ADD COLUMN IF NOT EXISTS resolved_at TIMESTAMPTZ;
