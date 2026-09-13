// Missing counters from an older server must not look like measured zeroes.
export function formatMetric(value) {
  return Number.isSafeInteger(value) && value >= 0 ? value.toLocaleString('zh-CN') : '—';
}
