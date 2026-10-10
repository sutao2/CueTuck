import { expect, it } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { parse } from '@vue/compiler-sfc';
import { parse as parseTemplate } from '@vue/compiler-dom';
import { parseExpression } from '@babel/parser';
import english from './i18n/en.json';

it('has no untranslated static UI text and resolves literal translation calls', () => {
  const files = readdirSync('src/components').filter(name => name.endsWith('.vue')).map(name => `src/components/${name}`);
  files.push('src/LauncherApp.vue', '../shared/IdentityForm.vue');
  const missing = [];
  for (const file of files) {
    const { descriptor } = parse(readFileSync(resolve(file), 'utf8'));
    const checkExpression = value => {
      let expression;
      try { expression = parseExpression(value); } catch { return; }
      const walk = node => {
        if (!node || typeof node !== 'object') return;
        if (node.type === 'CallExpression' && node.callee.name === 'tr' && node.arguments[0]?.type === 'StringLiteral') {
          const message = node.arguments[0].value;
          if (!Object.hasOwn(english, message)) missing.push(`${file}: missing ${message}`);
        }
        for (const [key, child] of Object.entries(node)) {
          if (['loc','extra','comments'].includes(key)) continue;
          if (Array.isArray(child)) child.forEach(walk);
          else if (child && typeof child === 'object') walk(child);
        }
      };
      walk(expression);
    };
    const visit = node => {
      if (node.type === 2 && /\p{Script=Han}/u.test(node.content)) missing.push(`${file}: static ${node.content}`);
      if (node.type === 6 && ['title','description','placeholder','alt','aria-label'].includes(node.name) && /\p{Script=Han}/u.test(node.value?.content ?? '')) missing.push(`${file}: ${node.name}`);
      if (node.type === 5) checkExpression(node.content.content);
      if (node.type === 7 && node.exp) checkExpression(node.exp.content);
      node.children?.forEach(visit);
      node.props?.forEach(visit);
    };
    visit(parseTemplate(descriptor.template.content));
  }
  expect(missing).toEqual([]);
});
