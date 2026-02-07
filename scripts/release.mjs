#!/usr/bin/env node

/**
 * Usage: node scripts/release.mjs <version>
 * Example: node scripts/release.mjs 0.3.0
 *
 * 1. Bump version in package.json, Cargo.toml, tauri.conf.json
 * 2. git add -u && git commit
 * 3. git tag v<version>
 * 4. git push && git push --tags
 */

import { execSync } from "child_process";
import { resolve } from "path";

const newVersion = process.argv[2];
if (!newVersion) {
  console.error("Usage: node scripts/release.mjs <version>");
  console.error("Example: node scripts/release.mjs 0.3.0");
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error(`Invalid version format: ${newVersion}`);
  console.error("Expected: MAJOR.MINOR.PATCH (e.g. 0.3.0)");
  process.exit(1);
}

const root = resolve(import.meta.dirname, "..");

function run(cmd) {
  console.log(`$ ${cmd}`);
  execSync(cmd, { cwd: root, stdio: "inherit" });
}

// 1. Bump version
run(`node scripts/bump-version.mjs ${newVersion}`);

// 2. Commit
run(`git add -u`);
run(`git commit -m "chore: bump version to ${newVersion}"`);

// 3. Tag
run(`git tag v${newVersion}`);

// 4. Push
run(`git push`);
run(`git push --tags`);

console.log();
console.log(`v${newVersion} released! GitHub Actions will create the release.`);
