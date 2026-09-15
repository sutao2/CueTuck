import { expect, it } from 'vitest';
import { squarePromptForUse } from './squarePromptUse.js';

it('accepts only ready translations of the current snapshot and respects original language', () => {
  const item = { id: 'remote', content: 'Original', asset_count: 3, translations: { zh: { status: 'ready', version: { source: { content: 'Original' }, content: '中文' } } } };
  expect(squarePromptForUse(item)).toMatchObject({ content: '中文', remote: true, asset_count: 0 });
  expect(squarePromptForUse(item, 'original').content).toBe('Original');
  expect(squarePromptForUse({ ...item, content: 'Changed' }).content).toBe('Changed');
  item.translations.zh.status = 'pending';
  expect(squarePromptForUse(item).content).toBe('Original');
  expect(item.asset_count).toBe(3);
});
