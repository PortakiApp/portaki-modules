-- pre-arrival-form @ schema v2 — guest count / special needs / ID document

ALTER TABLE module_pre_arrival_form.pre_arrival_response
    ADD COLUMN IF NOT EXISTS guest_count TEXT;

ALTER TABLE module_pre_arrival_form.pre_arrival_response
    ADD COLUMN IF NOT EXISTS special_needs TEXT;

ALTER TABLE module_pre_arrival_form.pre_arrival_response
    ADD COLUMN IF NOT EXISTS id_document TEXT;
