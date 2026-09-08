import { it, expect } from 'vitest';
import { referenceImages, referenceLink } from './squareReference.js';
it('allows reviewed HTTPS image URLs only', () => {
  expect(referenceImages({reference:{images:['https://cms-assets.youmind.com/media/one.jpg','http://cms-assets.youmind.com/a','https://cms-assets.youmind.com.evil/a','javascript:alert(1)','https://u:p@cms-assets.youmind.com/a']}})).toEqual(['https://cms-assets.youmind.com/media/one.jpg']);
  expect(referenceImages({})).toEqual([]);
  expect(referenceImages({reference:{images:'invalid'}})).toEqual([]);
  expect(referenceLink('javascript:alert(1)')).toBe('');
  expect(referenceLink('https://github.com/f/prompts.chat')).toContain('github.com');
});
