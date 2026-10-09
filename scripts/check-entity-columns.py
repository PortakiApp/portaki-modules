#!/usr/bin/env python3
"""Chaque champ d'une entité a sa colonne dans les migrations du module.

La plateforme écrit une ligne d'entité avec une colonne par champ de la structure
`#[portaki_sdk::entity]`, mais la table, elle, ne vient que de `db/migrations/*.sql`. Un champ
ajouté à la structure sans `ALTER TABLE … ADD COLUMN` fait échouer toutes les écritures : c'est
ce qui a cassé lost-found, consumables et pre-arrival-form du 4 au 9 octobre 2026, sans qu'aucun
test ne le voie (les mocks n'ont pas de table).

Une colonne compte dès que son nom apparaît comme mot dans une migration du module : grossier,
mais il ne laisse passer un oubli que si le nom traîne ailleurs dans le SQL.
"""
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
ENTITY = re.compile(
    r"#\[portaki_sdk::entity[^\]]*\]\s*(?:#\[[^\]]*\]\s*)*pub struct (\w+)\s*\{(.*?)\n\}", re.S
)
FIELD = re.compile(r"^\s*pub (\w+)\s*:", re.M)

missing = []
for module in sorted((ROOT / "modules").iterdir()):
    sources = list((module / "src").rglob("*.rs")) if (module / "src").is_dir() else []
    entities = [m for path in sources for m in ENTITY.finditer(path.read_text())]
    if not entities:
        continue
    sql = " ".join(p.read_text() for p in sorted((module / "db" / "migrations").glob("*.sql"))).lower()
    for name, body in ((m.group(1), m.group(2)) for m in entities):
        for field in FIELD.findall(body):
            if not re.search(rf"\b{re.escape(field)}\b", sql):
                missing.append(f"{module.name}: {name}.{field} n'a pas de colonne dans db/migrations")

if missing:
    print("\n".join(missing), file=sys.stderr)
    print("ajoutez une migration (ALTER TABLE … ADD COLUMN IF NOT EXISTS) et montez schema_version", file=sys.stderr)
    sys.exit(1)
print("chaque champ d'entité a sa colonne")
