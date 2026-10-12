-- issue-report @ schema v2 — idempotent (module_issue_report)
--
-- status: new · in_progress · resolved. Rows written before it read resolved when resolved_at is
-- set, new otherwise.
-- guest_name, stay_check_in, stay_check_out: the guest and the stay's dates at submit time.
-- issue_report_event: each report's history (sent, status changed, reply sent).

ALTER TABLE module_issue_report.issue_report
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT '';

UPDATE module_issue_report.issue_report
    SET status = CASE WHEN resolved_at IS NULL THEN 'new' ELSE 'resolved' END
    WHERE status = '';

ALTER TABLE module_issue_report.issue_report
    ADD COLUMN IF NOT EXISTS guest_name TEXT NOT NULL DEFAULT '';

ALTER TABLE module_issue_report.issue_report
    ADD COLUMN IF NOT EXISTS stay_check_in TIMESTAMPTZ;

ALTER TABLE module_issue_report.issue_report
    ADD COLUMN IF NOT EXISTS stay_check_out TIMESTAMPTZ;

CREATE TABLE IF NOT EXISTS module_issue_report.issue_report_event (
    id UUID PRIMARY KEY,
    report_id UUID NOT NULL,
    stay_id UUID NOT NULL,
    kind TEXT NOT NULL,
    actor TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT '',
    text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    property_id UUID NOT NULL
);

CREATE INDEX IF NOT EXISTS issue_report_event_report_idx
    ON module_issue_report.issue_report_event (report_id);

CREATE INDEX IF NOT EXISTS issue_report_event_stay_idx
    ON module_issue_report.issue_report_event (stay_id);

CREATE INDEX IF NOT EXISTS issue_report_event_property_idx
    ON module_issue_report.issue_report_event (property_id);
