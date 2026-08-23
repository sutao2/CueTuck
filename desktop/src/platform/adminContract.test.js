import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { listAdminOperations } from "./adminContract.js";

const openapiPath = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/reference/openapi/admin.yaml",
);

const REQUIRED = [
  { method: "GET", path: "/v1/admin/me", auth: "admin" },
  { method: "GET", path: "/v1/admin/publications", auth: "admin" },
  { method: "POST", path: "/v1/admin/publications/{id}/approve", auth: "admin" },
  { method: "POST", path: "/v1/admin/publications/{id}/reject", auth: "admin" },
  { method: "GET", path: "/v1/admin/users", auth: "admin" },
  { method: "GET", path: "/v1/admin/settings", auth: "admin" },
  { method: "PUT", path: "/v1/admin/settings", auth: "admin" },
];

describe("admin OpenAPI mapping", () => {
  it("lists every contract path with admin auth", () => {
    const yaml = readFileSync(openapiPath, "utf8");
    const operations = listAdminOperations(yaml);
    expect(operations).toEqual(REQUIRED);
  });
});
