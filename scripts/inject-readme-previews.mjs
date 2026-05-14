/**
 * Insère la section « Aperçu (illustration) » dans les README des modules portaki-modules
 * si absente. À lancer depuis la racine du dépôt portaki-modules :
 *   node scripts/inject-readme-previews.mjs
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const modulesDir = path.join(__dirname, "../modules");

function previewBlock(svgId) {
  return `## Aperçu (illustration)

> Rendu **factice** pour la documentation — aligné sur la maquette [\`guest-modules-section.jsx\`](../../portaki-web/public/design-handoff/guest-modules-section.jsx), pas une capture du build npm actuel.

<p align="center">
  <img src="../../../portaki-web/public/module-previews/${svgId}.svg" width="220" alt="Aperçu factice du module côté voyageur" />
</p>

`;
}

for (const name of fs.readdirSync(modulesDir, { withFileTypes: true })) {
  if (!name.isDirectory()) {
    continue;
  }
  const readmePath = path.join(modulesDir, name.name, "README.md");
  if (!fs.existsSync(readmePath)) {
    continue;
  }
  let s = fs.readFileSync(readmePath, "utf8");
  if (s.includes("module-previews/")) {
    console.log("skip (already)", name.name);
    continue;
  }
  const block = previewBlock(name.name);
  if (s.includes("> 🎯 **En une phrase**")) {
    s = s.replace("---\n\n> 🎯 **En une phrase**", `---\n\n${block}> 🎯 **En une phrase**`);
  } else if (s.includes("> **En une phrase**")) {
    s = s.replace("---\n\n> **En une phrase**", `---\n\n${block}> **En une phrase**`);
  } else if (name.name === "ical-sync") {
    s = s.replace("# @portaki/module-ical-sync\n\n", `# @portaki/module-ical-sync\n\n${block}`);
  } else {
    console.warn("skip (pattern)", name.name);
    continue;
  }
  fs.writeFileSync(readmePath, s, "utf8");
  console.log("patched", name.name);
}
