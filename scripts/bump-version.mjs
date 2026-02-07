#!/usr/bin/env node

/**
 * Usage: node scripts/bump-version.mjs <version>
 * Example: node scripts/bump-version.mjs 0.2.0
 *
 * Updates version in:
 *   - package.json
 *   - src-tauri/Cargo.toml
 *   - src-tauri/tauri.conf.json
 */

import { readFileSync, writeFileSync } from "fs";
import { resolve } from "path";

const newVersion = process.argv[2];
if (!newVersion) {
  console.error("Usage: node scripts/bump-version.mjs <version>");
  console.error("Example: node scripts/bump-version.mjs 0.2.0");
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error(`Invalid version format: ${newVersion}`);
  console.error("Expected: MAJOR.MINOR.PATCH (e.g. 0.2.0)");
  process.exit(1);
}

const root = resolve(import.meta.dirname, "..");

// package.json
const pkgPath = resolve(root, "package.json");
const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"));
const oldVersion = pkg.version;
pkg.version = newVersion;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");

// src-tauri/tauri.conf.json
const tauriConfPath = resolve(root, "src-tauri/tauri.conf.json");
const tauriConf = JSON.parse(readFileSync(tauriConfPath, "utf-8"));
tauriConf.version = newVersion;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");

// src-tauri/Cargo.toml
const cargoPath = resolve(root, "src-tauri/Cargo.toml");
let cargo = readFileSync(cargoPath, "utf-8");
cargo = cargo.replace(
  /^version = ".*"$/m,
  `version = "${newVersion}"`,
);
writeFileSync(cargoPath, cargo);

console.log(`${oldVersion} -> ${newVersion}`);
console.log(`  package.json`);
console.log(`  src-tauri/tauri.conf.json`);
console.log(`  src-tauri/Cargo.toml`);
console.log();
console.log(`Next steps:`);
console.log(`  git add -u && git commit -m "chore: bump version to ${newVersion}"`);
console.log(`  git tag v${newVersion}`);
console.log(`  git push && git push --tags`);
