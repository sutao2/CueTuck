import { afterEach, expect, it, vi } from "vitest";
import { loginSession, resetMemorySession, setSessionTransport } from "./session.js";
import { resetBilling, startBillingCheckout } from "./billing.js";

afterEach(() => { vi.unstubAllGlobals(); resetMemorySession(); resetBilling(); });

it("posts mock outcome and reports server errors rather than fake unpaid status", async () => {
  setSessionTransport(async () => ({ email: "test@example.test", access_token: "acc.test" }));
  await loginSession({ email: "test@example.test", password: "test" });
  const request = vi.fn(async () => ({ ok: true, status: 200, json: async () => ({ mock: true, mock_pro: true, pro: false }) }));
  vi.stubGlobal("fetch", request);
  expect(await startBillingCheckout("success")).toMatchObject({ mock: true, pro: false });
  expect(JSON.parse(request.mock.calls[0][1].body)).toEqual({ mock_outcome: "success" });
  request.mockResolvedValueOnce({ ok: false, status: 502, json: async () => ({}) });
  await expect(startBillingCheckout("success")).rejects.toThrow("支付请求失败");
});
