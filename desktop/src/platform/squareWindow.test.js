import { expect, it } from 'vitest';
import { buildRows, visibleRange } from './squareWindow.js';

it('bounds mounted rows across 20k items, mixed covers, list mode and column changes', () => {
  const items = Array.from({ length: 20000 }, (_, i) => ({ reference: i % 5 === 0 ? { images: ['cover'] } : null }));
  for (const columns of [1, 2, 3, 6]) for (const list of [false, true]) {
    const rows = buildRows(items, columns, list, 14);
    for (const y of [0, 3000, 100000, rows.at(-1).top]) {
      const range = visibleRange(rows, y, 900, columns, items.length);
      expect(range.end - range.start).toBeLessThanOrEqual(14 * columns);
      expect(range.start % columns).toBe(0);
      const last = rows[Math.ceil(range.end / columns) - 1];
      expect(range.top + (last.top + last.height - range.top) + range.bottom).toBe(rows.at(-1).top + rows.at(-1).height);
    }
  }
});

it('handles empty data and clamps to the final incomplete row', () => {
  expect(visibleRange([], 0, 720, 3, 0)).toEqual({ start: 0, end: 0, top: 0, bottom: 0 });
  const rows = buildRows(Array.from({ length: 49 }, () => ({})), 3, false, 14);
  expect(visibleRange(rows, rows.at(-1).top, 720, 3, 49).end).toBe(49);
});

it('reserves image and attachment space for published covers before they load', () => {
  expect(buildRows([{preview_asset:{id:"image"}}],1,false,14)[0].height).toBe(460);
  expect(buildRows([{preview_asset:{id:"image"}}],1,true,14)[0].height).toBe(132);
});

it('reserves collection media before its assets decode', () => {
  const rows = buildRows([{cover_assets:[{id:'cover'}]}, {}, {cover_json:'["cover"]'}], 1, false, 20);
  expect(rows[0].height).toBeGreaterThan(rows[1].height);
  expect(rows[2].height).toBe(rows[0].height);
  expect(rows[1].top).toBe(rows[0].height + 20);
});

it('does not reserve media for empty or invalid collection covers', () => {
  const rows = buildRows([{cover_json:'[]'}, {cover_json:'{}'}, {cover_json:'invalid'}], 1, false, 20);
  expect(rows.every(row => row.height === 280)).toBe(true);
});
