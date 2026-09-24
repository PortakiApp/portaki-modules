-- issue-report @ schema v1 — idempotent (module_issue_report)
--
-- resolved_at: host resolution timestamp (NULL = still open).
-- photo: optional guest photo reference (portaki-file:<uuid>).

CREATE SCHEMA IF NOT EXISTS module_issue_report;

CREATE TABLE IF NOT EXISTS module_issue_report.issue_report (
    id UUID PRIMARY KEY,
    stay_id UUID NOT NULL,
    category TEXT NOT NULL,
    summary TEXT NOT NULL,
    details TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    property_id UUID NOT NULL,
    resolved_at TIMESTAMPTZ,
    photo TEXT
);

CREATE INDEX IF NOT EXISTS issue_report_stay_idx
    ON module_issue_report.issue_report (stay_id);

CREATE INDEX IF NOT EXISTS issue_report_property_idx
    ON module_issue_report.issue_report (property_id);
