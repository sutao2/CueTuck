import { describe, expect, it, vi } from "vitest";
import { copyThenPaste } from "./paste.js";

describe("copyThenPaste", () => {
  it("keeps clipboard text when paste command fails", async () => {
    const writeText = vi.fn();
    const invoke = vi.fn().mockRejectedValue(new Error("no accessibility"));
    const result = await copyThenPaste("最终文本", { writeText, invoke });
    expect(writeText).toHaveBeenCalledWith("最终文本");
    expect(result.ok).toBe(false);
    expect(result.message).toMatch(/已复制，未能粘贴/);
  });
});
