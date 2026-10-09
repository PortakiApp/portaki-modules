-- consumables @ schema v4 — idempotent (module_consumables)
--
-- La réponse de l'hôte au voyageur, sous sa demande de réassort. Vide tant que l'hôte n'a rien
-- écrit. Le statut « planned » (Prévu) tient dans la colonne `status` existante.

ALTER TABLE module_consumables.consumable_report
    ADD COLUMN IF NOT EXISTS host_reply TEXT;
