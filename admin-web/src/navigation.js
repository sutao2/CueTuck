export const adminPages = ['overview', 'audit', 'system', 'review', 'content', 'users', 'billing', 'categories', 'models', 'reports', 'rules', 'moderation', 'ai-models', 'ai-skills', 'mail', 'notifications', 'identity', 'oauth', 'settings', 'security'];

export function requestedPage(hash = window.location.hash) {
  const value = hash.replace(/^#\/?/, '').split('?')[0];
  return adminPages.includes(value) ? value : 'review';
}

export function permittedPage(page, permissions = {}) {
  if (['overview', 'users', 'content', 'billing', 'categories', 'models', 'reports', 'rules'].includes(page) && !permissions.users) return false;
  if (['notifications', 'audit', 'system', 'oauth', 'settings', 'moderation', 'ai-models', 'ai-skills', 'mail', 'identity'].includes(page) && !permissions.configuration) return false;
  return adminPages.includes(page);
}
