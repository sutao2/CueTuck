import { expect, it, vi, afterEach } from 'vitest';
import { apiBase, normalizeApiBase } from '../../../shared/apiBase.js';
afterEach(() => vi.unstubAllEnvs());
it('normalizes a shared configured origin and defaults to loopback', () => {
  vi.stubEnv('VITE_API_BASE', 'https://api.example.test/'); expect(apiBase()).toBe('https://api.example.test');
  expect(normalizeApiBase()).toBe('http://127.0.0.1:8787');
  expect(normalizeApiBase('http://[::1]:8787/')).toBe('http://[::1]:8787');
});
it.each(['http://example.com', 'https://user:pass@example.com', 'file:///tmp/x', 'https://api.example/x', 'https://api.example?q=x', 'https://api.example#x', 'invalid'])('rejects unsafe or ambiguous API origin %s', value => {
  expect(() => normalizeApiBase(value)).toThrow();
});
