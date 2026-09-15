export const PAGE_SIZE = 24;
export async function requestJson(url, { signal, fetcher = fetch } = {}) {
  const response = await fetcher(url, { signal: signal ? AbortSignal.any([signal, AbortSignal.timeout(15000)]) : AbortSignal.timeout(15000), credentials: 'omit' });
  if (response.status === 409) throw Error('本轮推荐已过期，请换一批重新加载。');
  if (!response.ok) throw Error(response.status === 403 || response.status === 429 ? '来源访问受限，请稍后重试。' : `内容加载失败（${response.status}），请重试。`);
  return response.json();
}
export async function browsePrompts(base, filters = {}, options = {}) {
  const { query = '', category = '', model = '', sort = '推荐', language = 'zh', offset = 0, recommendation = null, exclude = [] } = filters;
  const params = new URLSearchParams({ q: query, category_id: category, model, sort, content_language: language, offset: String(offset), limit: String(PAGE_SIZE) });
  if (recommendation) params.set('recommendation', recommendation);
  if (exclude.length) params.set('exclude', JSON.stringify(exclude));
  const page = await requestJson(`${base}/v1/square/browse?${params}`, options);
  if (!Array.isArray(page.items) || page.items.length > PAGE_SIZE || !Number.isInteger(page.total) || page.total < 0 || (page.next_offset !== null && (!Number.isInteger(page.next_offset) || page.next_offset <= offset))) throw Error('广场分页响应无效，请重试。');
  return page;
}
export function categoryCount(id, categories, counts) {
  if (!counts) return null;
  const ids = new Set([id]);
  for (const category of categories) if (category.parent_id === id) ids.add(category.id);
  return [...ids].reduce((sum, key) => sum + (counts[key] || 0), 0);
}
export function safeImage(item, base = '') {
  const images = Array.isArray(item.reference?.images) ? item.reference.images : [];
  for (const value of images) {
    try { const url = new URL(value); if (url.protocol === 'https:' && url.hostname === 'cms-assets.youmind.com' && !url.username && !url.password && !url.port) return url.href; } catch {}
  }
  return item.preview_asset?.id ? `${base}/v1/square/items/${encodeURIComponent(item.id)}/assets/${encodeURIComponent(item.preview_asset.id)}` : '';
}
export function safeLink(value) {
  try { const url = new URL(value); return url.protocol === 'https:' && !url.username && !url.password ? url.href : ''; } catch { return ''; }
}
export async function skillCatalog(repo, options = {}) {
  if (!/^[\w.-]+\/[\w.-]+$/.test(repo)) throw Error('请选择公开来源列表中的仓库。');
  const base = `https://api.github.com/repos/${repo}`;
  const head = await requestJson(`${base}/commits/HEAD`, options);
  if (!/^[a-f0-9]{40}$/.test(head.sha)) throw Error('无法确认来源提交。');
  const tree = await requestJson(`${base}/git/trees/${head.sha}?recursive=1`, options);
  if (tree.truncated || !Array.isArray(tree.tree)) throw Error('来源目录不完整，请在 GitHub 查看。');
  const entries = tree.tree.filter(n => n.type === 'blob' && (n.path === 'SKILL.md' || n.path.endsWith('/SKILL.md'))).map(n => {
    const directory = n.path === 'SKILL.md' ? '' : n.path.slice(0, -9);
    return { id: `${repo}:${directory}`, kind: 'skill', repo, commit: head.sha, directory, path: n.path, title: directory.split('/').at(-1) || repo, description: '', model: 'Skill' };
  }).sort((a, b) => a.directory.localeCompare(b.directory));
  if (entries.length > 1000) throw Error('当前仓库超过 1,000 个 Skills，请前往来源查看。');
  return { repo, commit: head.sha, entries };
}
export async function readSkill(item, { signal, fetcher = fetch } = {}) {
  const path = item.path.split('/').map(encodeURIComponent).join('/');
  const response = await fetcher(`https://raw.githubusercontent.com/${item.repo}/${item.commit}/${path}`, { credentials: 'omit', signal: signal ? AbortSignal.any([signal, AbortSignal.timeout(15000)]) : AbortSignal.timeout(15000) });
  if (!response.ok) throw Error(`Skill 说明读取失败（${response.status}），可重试或查看来源。`);
  const body = await response.text();
  if (body.length > 2_000_000) throw Error('说明文件过大，请前往来源查看。');
  return body;
}
export function skillDescription(body) {
  const frontmatter = body.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!frontmatter) return '';
  const match = frontmatter[1].match(/^description:\s*(.*)(?:\r?\n((?:[ \t]+[^\n]*(?:\n|$))*))?/m);
  if (!match) return '';
  return [match[1].replace(/^[>|][-+]?$/, ''), match[2] || ''].join(' ').trim().replace(/^(["'])([\s\S]*)\1$/, '$2').replace(/\s+/g, ' ');
}
