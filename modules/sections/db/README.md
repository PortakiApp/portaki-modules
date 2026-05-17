# Schéma `sections`

- **Propriété** : module `sections` (`app.portaki.module.sections`).
- **Source de vérité** : `schema.sql` dans ce dossier.
- **Application aujourd’hui** : copie versionnée dans `portaki-api/.../db/migration/V58__module_sections_items.sql` (Flyway unique sur la base core).
- **Cible** : **Atlas** + `portaki-modules/tools/module-migrator` (révisions versionnées) — voir `portaki-internal-docs/MODULE_PLATFORM_PREPARATION.md`.

Ne pas ajouter d’entités JPA pour ce module dans `portaki-api` : persistance dans `backend/` uniquement.
