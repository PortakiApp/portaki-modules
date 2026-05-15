/**
 * Validates portaki.module.json in each modules/ subfolder against the JSON Schema
 * published from portaki-sdk (single source of truth).
 */
import Ajv from "ajv";
import addFormats from "ajv-formats";
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import * as lucide from "lucide-react";

const SCHEMA_URL =
    "https://raw.githubusercontent.com/PortakiApp/portaki-sdk/main/schema/module.v1.json";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "..");
const modulesDir = join(root, "modules");

const res = await fetch(SCHEMA_URL);
if (!res.ok) {
    console.error(`[validate-manifests] failed to fetch schema: ${res.status} ${SCHEMA_URL}`);
    process.exit(1);
}
const schema = await res.json();
const ajv = new Ajv({ allErrors: true, strict: false });
addFormats(ajv);
const validate = ajv.compile(schema);

const AGPLISH = new Set(["AGPL-3.0", "AGPL-3.0-only", "AGPL-3.0-or-later"]);

/** Licences additionnelles acceptées pour les modules communautaires (hors AGPL). */
const COMMUNITY_LICENSES = new Set([
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "MPL-2.0",
]);

function toPascalFromKebab(icon) {
    return icon
        .split("-")
        .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
        .join("");
}

function listModuleDirs() {
    return readdirSync(modulesDir, { withFileTypes: true })
        .filter((d) => d.isDirectory())
        .map((d) => d.name);
}

let failed = false;

for (const dir of listModuleDirs()) {
    const manifestPath = join(modulesDir, dir, "portaki.module.json");
    if (!existsSync(manifestPath)) {
        console.warn(`[skip] modules/${dir}/portaki.module.json missing`);
        continue;
    }
    const raw = readFileSync(manifestPath, "utf8");
    let data;
    try {
        data = JSON.parse(raw);
    } catch (e) {
        console.error(`[error] ${manifestPath}: invalid JSON`, e);
        failed = true;
        continue;
    }
    if (!validate(data)) {
        console.error(`[error] ${manifestPath}: schema`, validate.errors);
        failed = true;
        continue;
    }
    if (data.id !== dir) {
        console.error(`[error] ${manifestPath}: id "${data.id}" must equal folder "${dir}"`);
        failed = true;
    }
    const semver = /^\d+\.\d+\.\d+$/;
    if (!semver.test(data.version)) {
        console.error(`[error] ${manifestPath}: version must be semver X.Y.Z`);
        failed = true;
    }
    if (data.type === "official" && !AGPLISH.has(data.license)) {
        console.error(
            `[error] ${manifestPath}: official modules must use AGPL-3.0 (got ${data.license})`,
        );
        failed = true;
    }
    if (
        data.type === "community" &&
        !AGPLISH.has(data.license) &&
        !COMMUNITY_LICENSES.has(data.license)
    ) {
        console.error(
            `[error] ${manifestPath}: community license must be AGPL-3.0 or one of: ${[...COMMUNITY_LICENSES].sort().join(", ")} (got ${data.license})`,
        );
        failed = true;
    }
    if (data.type === "official" && data.author?.name !== "Portaki") {
        console.error(`[error] ${manifestPath}: official modules require author.name === "Portaki"`);
        failed = true;
    }
    const pascal = toPascalFromKebab(data.icon);
    if (!lucide[pascal]) {
        console.error(
            `[error] ${manifestPath}: icon "${data.icon}" → "${pascal}" not found in lucide-react`,
        );
        failed = true;
    }
}

if (failed) {
    process.exit(1);
}
console.log("All portaki.module.json files validated.");
