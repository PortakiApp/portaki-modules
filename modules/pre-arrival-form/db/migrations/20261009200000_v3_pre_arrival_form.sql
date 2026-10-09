-- pre-arrival-form @ schema v3 — idempotent (module_pre_arrival_form)
--
-- Les réponses aux questions de l'hôte, en JSON : la question telle qu'elle était posée, et la
-- réponse.

ALTER TABLE module_pre_arrival_form.pre_arrival_response
    ADD COLUMN IF NOT EXISTS custom_answers TEXT;
