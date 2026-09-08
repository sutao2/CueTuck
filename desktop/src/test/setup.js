import { beforeEach, afterEach, vi } from 'vitest';

// Unit tests must behave the same whether a developer API happens to be running
// or not. Individual HTTP tests replace this with their own explicit response.
beforeEach(() => {
  vi.stubGlobal('fetch', vi.fn(async () => { throw Error('Network disabled in unit tests; use an explicit transport'); }));
});
afterEach(() => vi.unstubAllGlobals());
