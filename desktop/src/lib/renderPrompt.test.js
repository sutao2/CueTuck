import { describe, expect, it } from "vitest";
import { extractVariables, renderPrompt } from "./renderPrompt.js";

describe("renderPrompt", () => {
  it("fills the three anonymous placeholders from the reported screenshot independently", () => {
    const content = "Sql {} dejk fer {}. hdjjf dev {} jhdfhk sd";
    expect(extractVariables(content)).toEqual(["占位符 1", "占位符 2", "占位符 3"]);
    expect(renderPrompt(content, { "占位符 1": "A", "占位符 2": "B", "占位符 3": "C" })).toBe("Sql A dejk fer B. hdjjf dev C jhdfhk sd");
    expect(renderPrompt(content, { "占位符 1": "A", "占位符 2": "", "占位符 3": "C" })).toBe("Sql A dejk fer {}. hdjjf dev C jhdfhk sd");
  });

  it("keeps blank slots unchanged, preserves literal values and avoids named-variable collisions", () => {
    const content = "{} {{占位符 1}} { } {{产品}} {{产品}}";
    expect(extractVariables(content)).toEqual(["占位符 2", "占位符 1", "占位符 3", "产品"]);
    expect(renderPrompt(content, { "占位符 2": "$&\n{{原样}}", "产品": "方舟" })).toBe("$&\n{{原样}} {{占位符 1}} { } 方舟 方舟");
  });

  it("preserves code, nested JSON, quoted/escaped braces and empty double braces", () => {
    const content = '示例 `const x = {}`、{"nested": {}}、"{}"、\\{}、{{}}、{{ }}\n```js\nfunction x() {}\n```\n~~~\n{}\n~~~\n填入 {}';
    expect(extractVariables(content)).toEqual(["占位符 1"]);
    expect(renderPrompt(content, { "占位符 1": "内容" })).toBe(content.replace("填入 {}", "填入 内容"));
    expect(renderPrompt("`{{名称}}`", { 名称: "显式变量" })).toBe("`显式变量`");
  });
  it("dedupes repeated variables", () => {
    expect(extractVariables("为 {{产品}} 写介绍，再次强调 {{产品}}")).toEqual(["产品"]);
  });

  it("keeps unfilled placeholders", () => {
    expect(renderPrompt("给 {{受众}} 的说明", {})).toBe("给 {{受众}} 的说明");
  });
});
