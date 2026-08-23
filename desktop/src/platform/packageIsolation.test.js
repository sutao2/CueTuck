import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const desktopRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

describe("desktop package isolation", () => {
  it("does not depend on or bundle admin-web", () => {
    const pkg = JSON.parse(readFileSync(resolve(desktopRoot, "package.json"), "utf8"));
    const deps = { ...pkg.dependencies, ...pkg.devDependencies };
    expect(Object.keys(deps).some((name) => name.includes("admin-web"))).toBe(false);
    expect(JSON.stringify(pkg)).not.toContain("admin-web");
    const tauri = readFileSync(resolve(desktopRoot, "src-tauri/tauri.conf.json"), "utf8");
    expect(tauri).not.toContain("admin-web");
    expect(JSON.parse(tauri).build.frontendDist).toBe("../dist");
  });
});
