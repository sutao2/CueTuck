import { describe, expect, it, vi } from "vitest";
import { copyThenPaste } from "./paste.js";

describe("copyThenPaste", () => {
  it("never invokes native paste when clipboard writing fails", async () => {
    const invoke = vi.fn();
    await expect(copyThenPaste("text", { writeText: async () => { throw new Error("denied"); }, invoke })).rejects.toThrow("denied");
    expect(invoke).not.toHaveBeenCalled();
  });
  it("keeps clipboard text when paste command fails", async () => {
    const writeText = vi.fn();
    const invoke = vi.fn().mockRejectedValue(new Error("no accessibility"));
    const result = await copyThenPaste("最终文本", { writeText, invoke });
    expect(writeText).toHaveBeenCalledWith("最终文本");
    expect(result.ok).toBe(false);
    expect(result.message).toMatch(/已复制，未能粘贴/);
    expect(result.message).toContain("no accessibility");
  });
  it("explains the browser limitation after copying", async () => {
    const writeText = vi.fn();
    const result = await copyThenPaste("浏览器样例", { writeText });
    expect(writeText).toHaveBeenCalledWith("浏览器样例");
    expect(result.ok).toBe(false);
    expect(result.message).toContain("浏览器预览不支持");
  });
});
