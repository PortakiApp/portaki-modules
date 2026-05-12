/**
 * Bump la version dans chaque package publishable sous modules/.
 * Usage:
 *   node scripts/bump-workspace-versions.mjs ci-run 42
 *     → major.minor.<run> depuis la base semver du package.json (ex. 0.1.42)
 *   node scripts/bump-workspace-versions.mjs prerelease develop.42
 *   node scripts/bump-workspace-versions.mjs set 0.2.0
 */
import fs from "node:fs";
import path from "node:path";

const modulesDir = "modules";

function discoverPublishableRoots() {
    const roots = [];
    for (const name of fs.readdirSync(modulesDir)) {
        const base = path.join(modulesDir, name);
        if (!fs.statSync(base).isDirectory()) continue;
        const rootPkg = path.join(base, "package.json");
        if (fs.existsSync(rootPkg)) {
            roots.push(base);
        }
        const fePkg = path.join(base, "frontend", "package.json");
        if (fs.existsSync(fePkg)) {
            roots.push(path.join(base, "frontend"));
        }
    }
    return roots.sort();
}

const roots = discoverPublishableRoots();

const [mode, arg] = process.argv.slice(2);

function baseVersion(current) {
    return current.replace(/[-+].*$/, "");
}

function writeVersion(root, version) {
    const p = path.join(root, "package.json");
    const j = JSON.parse(fs.readFileSync(p, "utf8"));
    j.version = version;
    fs.writeFileSync(p, `${JSON.stringify(j, null, 2)}\n`);
}

if (mode === "ci-run") {
    const run = arg;
    if (!run || !/^\d+$/.test(run)) {
        throw new Error("usage: ci-run <run_number> (entier)");
    }
    for (const root of roots) {
        const p = path.join(root, "package.json");
        const j = JSON.parse(fs.readFileSync(p, "utf8"));
        const base = baseVersion(j.version);
        const parts = base.split(".");
        const major = parts[0];
        const minor = parts[1];
        if (major == null || minor == null) {
            throw new Error(`Impossible de lire major.minor depuis "${base}" (${p})`);
        }
        writeVersion(root, `${major}.${minor}.${run}`);
    }
} else if (mode === "prerelease") {
    if (!arg) {
        throw new Error("usage: prerelease <suffixe> ex. develop.42");
    }
    for (const root of roots) {
        const p = path.join(root, "package.json");
        const j = JSON.parse(fs.readFileSync(p, "utf8"));
        const base = baseVersion(j.version);
        writeVersion(root, `${base}-${arg}`);
    }
} else if (mode === "set") {
    if (!arg) {
        throw new Error("usage: set <semver>");
    }
    for (const root of roots) {
        writeVersion(root, arg);
    }
} else {
    console.error("Usage: ci-run <run_number> | prerelease <suffixe> | set <semver>");
    process.exit(1);
}
