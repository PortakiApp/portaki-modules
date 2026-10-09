-- pre-arrival-form @ schema v2 — idempotent (module_pre_arrival_form)
--
-- Le moyen de transport était déclaré sur l'entité sans colonne : chaque réponse au formulaire
-- échouait à l'écriture.

ALTER TABLE module_pre_arrival_form.pre_arrival_response
    ADD COLUMN IF NOT EXISTS transport TEXT;
