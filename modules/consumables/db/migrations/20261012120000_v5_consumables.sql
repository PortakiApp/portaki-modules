-- consumables @ schema v5 — idempotent (module_consumables)
--
-- « Où le trouver » d'un produit (carte JSON des langues, vide par défaut), et, sur une demande,
-- le voyageur et les dates du séjour relevés à l'envoi : la table des demandes de l'hôte les
-- montre sans rappeler la plateforme. Les demandes d'avant restent sans ces colonnes remplies.

ALTER TABLE module_consumables.consumable_item
    ADD COLUMN IF NOT EXISTS location TEXT NOT NULL DEFAULT '';

ALTER TABLE module_consumables.consumable_report
    ADD COLUMN IF NOT EXISTS guest_name TEXT NOT NULL DEFAULT '';

ALTER TABLE module_consumables.consumable_report
    ADD COLUMN IF NOT EXISTS stay_check_in TIMESTAMPTZ;

ALTER TABLE module_consumables.consumable_report
    ADD COLUMN IF NOT EXISTS stay_check_out TIMESTAMPTZ;
