-- checklist @ schema v2 — idempotent (module_checklist)
--
-- Deux colonnes de plus par étape (§2.9) : la rubrique sous laquelle elle se range et la précision
-- qui l'accompagne. Chacune porte une carte de langues en JSON, comme `label_fr` le fait déjà —
-- vide quand l'hôte n'a rien écrit, ce qui est le cas de toutes les lignes existantes.

ALTER TABLE module_checklist.checklist_item
    ADD COLUMN IF NOT EXISTS group_i18n TEXT NOT NULL DEFAULT '';

ALTER TABLE module_checklist.checklist_item
    ADD COLUMN IF NOT EXISTS description_i18n TEXT NOT NULL DEFAULT '';
