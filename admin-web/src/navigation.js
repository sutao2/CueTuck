export const adminPages = ['overview', 'audit', 'system', 'review', 'content', 'users', 'billing', 'categories', 'models', 'reports', 'rules', 'moderation', 'ai-models', 'ai-skills', 'mail', 'notifications', 'identity', 'oauth', 'settings', 'security'];

export const adminNavGroups = [
  { label: '工作空间', items: [{ page: 'overview', icon: 'cyan' }, { page: 'users', icon: 'user' }, { page: 'billing', icon: 'file' }] },
  { label: '内容与目录', items: [{ page: 'review', icon: 'shield' }, { page: 'content', icon: 'square' }, { page: 'categories', icon: 'folder' }, { page: 'models', icon: 'models' }, { page: 'reports', icon: 'rose' }] },
  { label: '智能审核', items: [{ page: 'moderation', icon: 'settings' }, { page: 'ai-models', icon: 'models' }, { page: 'ai-skills', icon: 'blue' }, { page: 'rules', icon: 'shield' }] },
  { label: '站点与接入', items: [{ page: 'settings', icon: 'globe' }, { page: 'oauth', icon: 'user' }, { page: 'identity', icon: 'plus' }, { page: 'mail', icon: 'file' }, { page: 'notifications', icon: 'rose' }] },
  { label: '系统', items: [{ page: 'audit', icon: 'list' }, { page: 'system', icon: 'database' }, { page: 'security', icon: 'shield' }] },
];

export function requestedPage(hash = window.location.hash) {
  const value = hash.replace(/^#\/?/, '').split('?')[0];
  return adminPages.includes(value) ? value : 'review';
}

export function permittedPage(page, permissions = {}) {
  if (['overview', 'users', 'content', 'billing', 'categories', 'models', 'reports', 'rules'].includes(page) && !permissions.users) return false;
  if (['notifications', 'audit', 'system', 'oauth', 'settings', 'moderation', 'ai-models', 'ai-skills', 'mail', 'identity'].includes(page) && !permissions.configuration) return false;
  return adminPages.includes(page);
}
