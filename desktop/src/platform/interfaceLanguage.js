import { ref } from 'vue';
import english from './i18n/en.json';

export const interfaceLanguage = ref('zh');

export function applyInterfaceLanguage(value) {
  interfaceLanguage.value = value === 'en' ? 'en' : 'zh';
  document.documentElement.lang = interfaceLanguage.value === 'en' ? 'en' : 'zh-CN';
  document.title = interfaceLanguage.value === 'en' ? 'CueTuck' : '唤词';
}

const escapeRegex = value => value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const patterns = Object.entries(english).filter(([source]) => /\{\d+\}/.test(source)).sort(([a], [b]) => b.replace(/\{\d+\}/g, '').length - a.replace(/\{\d+\}/g, '').length).map(([source, translated]) => ({
  expression: new RegExp(`^${source.split(/\{\d+\}/).map(escapeRegex).join('([\\s\\S]*?)')}$`),
  translated,
}));
const interpolate = (message, values) => message.replace(/\{(\d+)\}/g, (token, index) => values[index] ?? token);

// Only app-owned UI messages enter this function. Interpolated titles, paths and
// content are preserved verbatim; unknown technical diagnostics remain readable.
export function tr(message, values) {
  if (typeof message !== 'string') return message;
  if (values) return interpolate(interfaceLanguage.value === 'en' ? (english[message] ?? message) : message, values);
  if (interfaceLanguage.value !== 'en') return message;
  if (Object.hasOwn(english, message)) return english[message];
  for (const { expression, translated } of patterns) {
    const match = expression.exec(message);
    if (match) return interpolate(translated, match.slice(1));
  }
  return message;
}

export function categoryLabel(category) {
  return category?.is_system ? tr(category.name) : (category?.name ?? "");
}
