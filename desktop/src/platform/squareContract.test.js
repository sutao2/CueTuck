import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { listSquareOperations } from "./squareContract.js";

const openapiPath = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../../docs/reference/openapi/square.yaml",
);

const REQUIRED = [
  { method: "GET", path: "/v1/square/items", auth: "none" },
  { method: "GET", path: "/v1/square/items/{id}", auth: "none" },
  { method: "GET", path: "/v1/square/items/{id}/content", auth: "none" },
  { method: "POST", path: "/v1/session", auth: "none" },
  { method: "DELETE", path: "/v1/session", auth: "user" },
  { method: "POST", path: "/v1/session/refresh", auth: "none" },
  { method: "GET", path: "/v1/favorites", auth: "user" },
  { method: "PUT", path: "/v1/favorites/{id}", auth: "user" },
  { method: "DELETE", path: "/v1/favorites/{id}", auth: "user" },
  { method: "POST", path: "/v1/publications", auth: "user" },
  { method: "GET", path: "/v1/health", auth: "none" },
  { method: "GET", path: "/v1/session/oauth/providers", auth: "none" },
  { method: "GET", path: "/v1/session/oauth/callback", auth: "none" },
  { method: "GET", path: "/v1/session/oauth/session/{flowId}", auth: "none" },
  { method: "GET", path: "/v1/session/oauth/{provider}", auth: "none" },
  { method: "POST", path: "/v1/media/upload", auth: "user" },
  { method: "GET", path: "/v1/media/{id}/url", auth: "none" },
];

describe("square OpenAPI mapping", () => {
  it("lists every contract path with the right auth", () => {
    const yaml = readFileSync(openapiPath, "utf8");
    const operations = listSquareOperations(yaml);
    expect(operations).toEqual(REQUIRED);
  });
});
