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
  { method: "GET", path: "/v1/admin/overview", auth: "admin" },
  { method: "POST", path: "/v1/admin/catalog/{kind}/{id}/migrate", auth: "admin" },
  { method: "GET", path: "/v1/admin/audit", auth: "admin" },
  { method: "GET", path: "/v1/admin/audit/export", auth: "admin" },
  { method: "GET", path: "/v1/admin/system", auth: "admin" },
  { method: "GET", path: "/v1/admin/me", auth: "admin" },
  { method: "GET", path: "/v1/admin/publications", auth: "admin" },
  { method: "POST", path: "/v1/admin/publications/{id}/approve", auth: "admin" },
  { method: "POST", path: "/v1/admin/publications/{id}/reject", auth: "admin" },
  { method: "GET", path: "/v1/admin/reviews", auth: "admin" },
  { method: "POST", path: "/v1/admin/reviews/batch", auth: "admin" },
  { method: "GET", path: "/v1/admin/content", auth: "admin" },
  { method: "GET", path: "/v1/admin/content/{id}", auth: "admin" },
  { method: "PUT", path: "/v1/admin/content/{id}", auth: "admin" },
  { method: "GET", path: "/v1/admin/users", auth: "admin" },
  { method: "GET", path: "/v1/admin/settings", auth: "admin" },
  { method: "PUT", path: "/v1/admin/settings", auth: "admin" },
  { method: "GET", path: "/v1/admin/oauth", auth: "admin" },
  { method: "PUT", path: "/v1/admin/oauth/{provider}", auth: "admin" },
  { method: "POST", path: "/v1/admin/oauth/{provider}/verify", auth: "admin" },
  { method: "GET", path: "/v1/admin/security", auth: "admin" },
  { method: "PUT", path: "/v1/admin/security/password", auth: "admin" },
  { method: "DELETE", path: "/v1/admin/security/sessions", auth: "admin" },
  { method: "GET", path: "/v1/admin/users/{email}", auth: "admin" },
  { method: "POST", path: "/v1/admin/users/{email}/actions", auth: "admin" },
  { method: "GET", path: "/v1/admin/catalog/{kind}", auth: "admin" },
  { method: "POST", path: "/v1/admin/catalog/{kind}", auth: "admin" },
  { method: "PUT", path: "/v1/admin/catalog/{kind}/{id}", auth: "admin" },
  { method: "DELETE", path: "/v1/admin/catalog/{kind}/{id}", auth: "admin" },
  { method: "GET", path: "/v1/admin/reports", auth: "admin" },
  { method: "GET", path: "/v1/admin/reports/export", auth: "admin" },
  { method: "GET", path: "/v1/admin/reports/{id}", auth: "admin" },
  { method: "PUT", path: "/v1/admin/reports/{id}", auth: "admin" },
  { method: "GET", path: "/v1/admin/safety-rules", auth: "admin" },
  { method: "PUT", path: "/v1/admin/safety-rules", auth: "admin" },
  { method: "POST", path: "/v1/admin/safety-rules/test", auth: "admin" },
  { method: "GET", path: "/v1/admin/moderation", auth: "admin" },
  { method: "PUT", path: "/v1/admin/moderation", auth: "admin" },
  { method: "GET", path: "/v1/admin/ai/config", auth: "admin" },
  { method: "PUT", path: "/v1/admin/ai/config", auth: "admin" },
  { method: "GET", path: "/v1/admin/ai/history", auth: "admin" },
  { method: "POST", path: "/v1/admin/ai/test", auth: "admin" },
  { method: "GET", path: "/v1/admin/mail/config", auth: "admin" },
  { method: "PUT", path: "/v1/admin/mail/config", auth: "admin" },
  { method: "POST", path: "/v1/admin/mail/test", auth: "admin" },
  { method: "GET", path: "/v1/admin/mail/deliveries", auth: "admin" },
  { method: "GET", path: "/v1/admin/mail/deliveries/{id}", auth: "admin" },
  { method: "POST", path: "/v1/admin/mail/deliveries/{id}/retry", auth: "admin" },
  { method: "GET", path: "/v1/admin/identity/policy", auth: "admin" },
  { method: "PUT", path: "/v1/admin/identity/policy", auth: "admin" },
  { method: "GET", path: "/v1/admin/identity/invitations", auth: "admin" },
  { method: "POST", path: "/v1/admin/identity/invitations", auth: "admin" },
  { method: "POST", path: "/v1/admin/identity/invitations/{id}/revoke", auth: "admin" },
  { method: "POST", path: "/v1/admin/users/{email}/password-reset", auth: "admin" },
  { method: "GET", path: "/v1/admin/site", auth: "admin" },
  { method: "PUT", path: "/v1/admin/site", auth: "admin" },
  { method: "GET", path: "/v1/admin/mock-billing/{kind}", auth: "admin" },
  { method: "POST", path: "/v1/admin/mock-billing/actions", auth: "admin" },
  { method: "GET", path: "/v1/admin/mock-billing/batches", auth: "admin" },
  { method: "POST", path: "/v1/admin/mock-billing/batches", auth: "admin" },
  { method: "GET", path: "/v1/admin/mock-billing/batches/{id}", auth: "admin" },
  { method: "PUT", path: "/v1/admin/mock-billing/batches/{id}", auth: "admin" },
  { method: "GET", path: "/v1/admin/notifications/config", auth: "admin" },
  { method: "PUT", path: "/v1/admin/notifications/config", auth: "admin" },
  { method: "POST", path: "/v1/admin/notifications/test", auth: "admin" },
  { method: "GET", path: "/v1/admin/notifications/deliveries", auth: "admin" },
  { method: "GET", path: "/v1/admin/notifications/deliveries/{id}", auth: "admin" },
  { method: "POST", path: "/v1/admin/notifications/deliveries/{id}/retry", auth: "admin" },
];

describe("admin OpenAPI mapping", () => {
  it("lists every contract path with admin auth", () => {
    const yaml = readFileSync(openapiPath, "utf8");
    const operations = listAdminOperations(yaml);
    expect(operations).toEqual(REQUIRED);
  });
});
