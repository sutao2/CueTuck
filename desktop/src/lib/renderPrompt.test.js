import { describe, expect, it } from "vitest";
import { extractVariables, renderPrompt, variableDefaults } from "./renderPrompt.js";

describe("renderPrompt", () => {
  it('recognizes imported advertisement arguments and uses defaults without rewriting the source',()=>{
    const source='创建一个 {argument name="aspect ratio" default="21:9 全景"} 格式。背景 {argument name="background color" default="产品主色调"}，手势 {argument name="hand gesture" default="做出射网手势的红色网纹手套"}。';
    expect(extractVariables(source)).toEqual(['aspect ratio','background color','hand gesture']);
    expect(variableDefaults(source)).toEqual({'aspect ratio':'21:9 全景','background color':'产品主色调','hand gesture':'做出射网手势的红色网纹手套'});
    expect(renderPrompt(source,{'background color':'蓝色'})).toBe('创建一个 21:9 全景 格式。背景 蓝色，手势 做出射网手势的红色网纹手套。');
    expect(source).toContain('{argument');
  });
  it('shares names and the first declared default with double braces, preserving literal inserted values',()=>{
    const source=`{{x}} {argument default='A {{nested}} {}' name='x'} {argument name="x" default="B"} {} {argument name="empty" default=""}`;
    expect(extractVariables(source)).toEqual(['x','占位符 1','empty']);
    expect(renderPrompt(source)).toBe('A {{nested}} {} A {{nested}} {} A {{nested}} {} {} ');
    expect(renderPrompt(source,{x:'$&\n{}'})).toBe('$&\n{} $&\n{} $&\n{} {} ');
    expect(renderPrompt('{argument name="x" default="A"}',{x:''})).toBe('A');
  });
  it('supports explicit arguments inside code strings, quotes and safe special names',()=>{
    const source=String.raw`const player = "{argument name="__proto__" default="A \"quote\""}"; {argument name='constructor'} {argument name="__v_isReactive" default="yes"}`;
    expect(extractVariables(source)).toEqual(['__proto__','constructor','__v_isReactive']);
    expect(Object.hasOwn(variableDefaults(source),'__proto__')).toBe(true);
    expect(renderPrompt(source)).toContain('A "quote"');
    expect(renderPrompt(source)).toContain("{argument name='constructor'}");
  });
  it('keeps invalid attributes and escaped declarations literal',()=>{
    for(const source of ['{argument name="" default="A"}','{argument name="x" name="y"}','{argument name="x" type="text"}','{argument default="A"}','{argument name="x" default=no}','{argument name="x"',String.raw`\{argument name="x" default="A"}`]) {
      expect(extractVariables(source)).toEqual([]);expect(renderPrompt(source)).toBe(source);
    }
  });
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
