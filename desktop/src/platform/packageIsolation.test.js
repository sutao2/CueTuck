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

  it("does not depend on or bundle web workbench", () => {
    const pkg = JSON.parse(readFileSync(resolve(desktopRoot, "package.json"), "utf8"));
    expect(JSON.stringify(pkg)).not.toContain("promptark-web");
    expect(JSON.stringify(pkg)).not.toContain("../web");
    const tauri = readFileSync(resolve(desktopRoot, "src-tauri/tauri.conf.json"), "utf8");
    expect(tauri).not.toContain("promptark-web");
    expect(tauri).not.toContain("../web");
    expect(JSON.parse(tauri).build.frontendDist).toBe("../dist");
  });

  it("keeps package version aligned with tauri and cargo", () => {
    const pkg = JSON.parse(readFileSync(resolve(desktopRoot, "package.json"), "utf8"));
    const tauri = JSON.parse(
      readFileSync(resolve(desktopRoot, "src-tauri/tauri.conf.json"), "utf8"),
    );
    const cargo = readFileSync(resolve(desktopRoot, "src-tauri/Cargo.toml"), "utf8");
    const cargoVersion = cargo.match(/^version = "([^"]+)"/m)?.[1];
    const lock = JSON.parse(readFileSync(resolve(desktopRoot, "package-lock.json"), "utf8"));
    expect(pkg.version).toBe(tauri.version);
    expect(pkg.version).toBe(cargoVersion);
    expect(lock.version).toBe(pkg.version);
    expect(lock.packages[""].version).toBe(pkg.version);
  });

  it("does not depend on or bundle the mcp server", () => {
    const pkg = JSON.parse(readFileSync(resolve(desktopRoot, "package.json"), "utf8"));
    expect(JSON.stringify(pkg)).not.toContain("promptark-mcp");
    expect(JSON.stringify(pkg)).not.toContain("../mcp");
    const tauri = readFileSync(resolve(desktopRoot, "src-tauri/tauri.conf.json"), "utf8");
    expect(tauri).not.toContain("promptark-mcp");
    expect(tauri).not.toContain("../mcp");
    expect(JSON.parse(tauri).build.frontendDist).toBe("../dist");
  });
});
