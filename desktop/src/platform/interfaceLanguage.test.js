import { afterEach, expect, it } from 'vitest';
import { computed } from 'vue';
import { applyInterfaceLanguage, categoryLabel, tr } from './interfaceLanguage.js';

afterEach(() => applyInterfaceLanguage('zh'));
it('updates existing reactive messages and preserves interpolated user content', () => {
  const text = computed(() => tr('已保存「{0}」。', ['设置']));
  expect(text.value).toBe('已保存「设置」。');
  applyInterfaceLanguage('en');
  expect(text.value).toBe('Saved “设置”.');
  expect(document.documentElement.lang).toBe('en');
  expect(tr('读取失败：原始诊断，请重试。')).toBe('Unable to load: 原始诊断. Please retry.');
  expect(tr('Unknown provider error')).toBe('Unknown provider error');
  applyInterfaceLanguage('zh');
  expect(text.value).toBe('已保存「设置」。');
});
it('translates preset category labels without changing custom names or identifiers', () => {
  applyInterfaceLanguage('en');
  const preset = { id: 'cat-software', name: '软件开发', is_system: true };
  expect(categoryLabel(preset)).toBe('Software development');
  expect(categoryLabel({ ...preset, is_system: false })).toBe('软件开发');
  expect(preset).toEqual({ id: 'cat-software', name: '软件开发', is_system: true });
});

it('keeps every interpolation token in the English catalog', async () => {
  const { default: english } = await import('./i18n/en.json');
  for (const [source, translated] of Object.entries(english)) {
    expect(translated.match(/\{\d+\}/g)?.sort() ?? [], source).toEqual(source.match(/\{\d+\}/g)?.sort() ?? []);
  }
});
