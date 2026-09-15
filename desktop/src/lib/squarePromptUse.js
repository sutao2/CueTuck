// Only ready versions matching this snapshot may replace the original template.
export function squarePromptForUse(item, language = 'zh') {
  const version = item.translations?.[language];
  const source = version?.version?.source?.content;
  const translated = version?.version?.content;
  const content = language !== 'original' && version?.status === 'ready'
    && (source === undefined || source === item.content) && typeof translated === 'string'
    ? translated : item.content;
  return { ...item, content, remote: true, asset_count: 0 };
}
