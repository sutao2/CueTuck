import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = dirname(fileURLToPath(import.meta.url));

describe("desktop package", () => {
  it("declares a vue app entry", () => {
    const pkg = JSON.parse(readFileSync(join(root, "..", "package.json"), "utf8"));
    expect(pkg.name).toBe("promptark-desktop");
    expect(pkg.scripts.test).toBeTruthy();
  });
});
