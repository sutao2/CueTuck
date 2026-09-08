// Imported references are data, never HTML. Only the reviewed image host is loaded.
export function referenceImages(item) {
  return (Array.isArray(item?.reference?.images) ? item.reference.images : [])
    .filter(url => {
      try { const u = new URL(url); return u.protocol === 'https:' && u.hostname === 'cms-assets.youmind.com' && !u.username && !u.password; }
      catch { return false; }
    }).slice(0, 6);
}
export function referenceLink(value) {
  try { const u = new URL(value); return u.protocol === 'https:' && !u.username && !u.password ? u.href : ''; }
  catch { return ''; }
}
