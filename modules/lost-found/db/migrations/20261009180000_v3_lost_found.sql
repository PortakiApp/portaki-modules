-- lost-found @ schema v3 — idempotent (module_lost_found)
--
-- Les champs du formulaire de la maquette (catégorie, pièce, photo, choix de retour) étaient
-- déclarés sur l'entité sans colonne : chaque écriture d'un signalement échouait.

ALTER TABLE module_lost_found.lost_found_report
    ADD COLUMN IF NOT EXISTS category TEXT,
    ADD COLUMN IF NOT EXISTS room TEXT,
    ADD COLUMN IF NOT EXISTS photo TEXT,
    ADD COLUMN IF NOT EXISTS return_choice TEXT;
