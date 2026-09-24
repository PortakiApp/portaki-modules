-- pre-arrival-form @ schema v1 — idempotent (module_pre_arrival_form)

CREATE SCHEMA IF NOT EXISTS module_pre_arrival_form;

CREATE TABLE IF NOT EXISTS module_pre_arrival_form.pre_arrival_response (
    id UUID PRIMARY KEY,
    stay_id UUID NOT NULL,
    property_id UUID NOT NULL,
    arrival_time TEXT,
    occasion TEXT,
    allergies TEXT,
    guest_message TEXT,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    guest_count TEXT,
    special_needs TEXT,
    id_document TEXT,
    CONSTRAINT pre_arrival_response_stay_uq UNIQUE (stay_id)
);

CREATE INDEX IF NOT EXISTS pre_arrival_response_stay_idx
    ON module_pre_arrival_form.pre_arrival_response (stay_id);
