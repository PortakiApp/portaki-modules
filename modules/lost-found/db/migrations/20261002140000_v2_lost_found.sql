-- lost-found @ schema v2 — idempotent (module_lost_found)
--
-- L'adresse de renvoi (§10). Demandée au voyageur seulement quand l'hôte propose le renvoi postal,
-- donc nulle sur toutes les lignes existantes : aucun signalement n'en portait avant ce champ.

ALTER TABLE module_lost_found.lost_found_report
    ADD COLUMN IF NOT EXISTS return_address TEXT;
