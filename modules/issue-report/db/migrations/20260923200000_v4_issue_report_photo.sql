-- issue-report @ schema v4 — optional guest photo reference (portaki-file:<uuid>).

ALTER TABLE module_issue_report.issue_report
    ADD COLUMN IF NOT EXISTS photo TEXT;
