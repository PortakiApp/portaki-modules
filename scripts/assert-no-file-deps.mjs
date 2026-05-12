#!/usr/bin/env node
/**
 * Refuse toute dépendance npm en `file:` dans les package.json des modules publiables.
 */
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const bad = [];

function scanDeps(path, label, obj) {
    if (!obj || typeof obj !== "object") return;
    for (const [k, v] of Object.entries(obj)) {
        if (typeof v === "string" && v.startsWith("file:")) {
            bad.push({ path, where: label, dep: k, value: v });
        }
    }
}

function checkPackageJson(path) {
    const raw = readFileSync(path, "utf8");
    const j = JSON.parse(raw);
    scanDeps(path, "dependencies", j.dependencies);
    scanDeps(path, "devDependencies", j.devDependencies);
    scanDeps(path, "optionalDependencies", j.optionalDependencies);
    scanDeps(path, "peerDependencies", j.peerDependencies);
}

const modulesDir = join(root, "modules");
for (const ent of readdirSync(modulesDir, { withFileTypes: true })) {
    if (!ent.isDirectory()) continue;
    const base = join(modulesDir, ent.name);
    const main = join(base, "package.json");
    if (existsSync(main)) checkPackageJson(main);
    const fe = join(base, "frontend", "package.json");
    if (existsSync(fe)) checkPackageJson(fe);
}

if (bad.length > 0) {
    console.error("[assert-no-file-deps] Interdit : dépendances file:");
    for (const b of bad) {
        console.error(`  ${b.path} → ${b.where}.${b.dep} = ${b.value}`);
    }
    process.exit(1);
}

console.log("[assert-no-file-deps] ok");
