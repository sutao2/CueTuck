// Fixed row geometry keeps image decoding and unmounting cards from moving the scroll position.
export function buildRows(items, columns, list, gap) {
  const rows = [];
  let top = 0;
  for (let start = 0; start < items.length; start += columns) {
    const cover = items.slice(start, start + columns).some(item => item.reference?.images?.[0]);
    const height = list ? 116 : cover ? 410 : 256;
    rows.push({ top, height });
    top += height + gap;
  }
  return rows;
}

export function visibleRange(rows, scroll, viewport, columns, count) {
  if (!rows.length) return { start: 0, end: 0, top: 0, bottom: 0 };
  // Binary search avoids a per-scroll scan over every loaded card.
  const find = y => {
    let lo = 0, hi = rows.length - 1;
    while (lo < hi) { const mid = (lo + hi) >>> 1; if (rows[mid].top + rows[mid].height < y) lo = mid + 1; else hi = mid; }
    return lo;
  };
  const first = Math.max(0, find(scroll) - 2), last = Math.min(rows.length - 1, find(scroll + viewport) + 2);
  const end = rows.at(-1);
  return { start: first * columns, end: Math.min(count, (last + 1) * columns), top: rows[first].top, bottom: end.top + end.height - rows[last].top - rows[last].height };
}
