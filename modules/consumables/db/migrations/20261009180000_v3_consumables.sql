-- consumables @ schema v3 — idempotent (module_consumables)
--
-- L'emoji d'un produit était déclaré sur l'entité sans colonne : chaque enregistrement du
-- catalogue échouait. Vide par défaut, comme le lit le module.

ALTER TABLE module_consumables.consumable_item
    ADD COLUMN IF NOT EXISTS emoji TEXT NOT NULL DEFAULT '';
