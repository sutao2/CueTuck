import sources from '../data/skill-sources.json';

export const skillCategories = [
  { id: '', name: '全部 Skills', icon: 'square' },
  { id: 'development', name: '软件开发', icon: 'blue' },
  { id: 'ai', name: 'AI 与模型', icon: 'models' },
  { id: 'cloud', name: '云服务与部署', icon: 'globe' },
  { id: 'security', name: '安全与审计', icon: 'shield' },
  { id: 'office', name: '办公与写作', icon: 'file' },
  { id: 'design', name: '设计与创作', icon: 'image' },
  { id: 'marketing', name: '营销与增长', icon: 'rose' },
  { id: 'science', name: '科研与数据', icon: 'cyan' },
  { id: 'tools', name: '日常工具', icon: 'settings' },
  { id: 'uncategorized', name: '未分类', icon: 'folder' },
];

// Match the Skill's own name/path, not trigger instructions in its description.
// Repository categories are fallbacks, never a claim about client compatibility.
const rules = [
  ['office', /\bartifact template\b/],
  ['development', /\b(api|system|database|software|architecture) design\b/],
  ['security', /\b(security|audit|vulnerability|secure|semgrep|codeql|fuzzing|cryptography|penetration)\b|安全|漏洞|审计/],
  ['marketing', /\b(marketing|seo|copywriting|campaign|conversion|ads|ad creative|ab testing|growth|social media|sales|pricing|copy editing|content strategy)\b|营销|增长|广告|转化/],
  ['science', /\b(scientific|science|research|bioinformatics|biology|chemistry|physics|statistics|data analysis|pandas|numpy|scipy|matplotlib|genomics)\b|科研|科学|统计|数据分析/],
  ['design', /\b(design|canvas|art|image|video|animation|remotion|gif|theme|brand|figma|illustration)\b|设计|绘画|图片|视频|创作/],
  ['office', /\b(docx|pptx|xlsx|pdf|document|spreadsheet|presentation|writing|blog|newsletter|internal comms|meeting|notes|documents|spreadsheets|presentations|report|memo|memorandum|budget)\b|办公|文档|写作|演示|表格|会议/],
  ['ai', /\b(ai|agentic|agent harness|autonomous loops|claude|codex|skill creator|prompt|llm|model training|fine tuning|inference|huggingface|transformers|diffusers|embeddings|rag|claude api|openai api|prompt engineering|mcp)\b|大模型|微调|机器学习|提示词/],
  ['cloud', /\b(azure|cloudflare|aws|gcp|deploy|deployment|docker|kubernetes|terraform|workers)\b|云服务|部署|容器/],
  ['development', /\b(react|nextjs|next js|vue|frontend|backend|typescript|javascript|python|rust|golang|code|coding|debugging|debug|build|testing|tests|test|tdd|swift|swiftui|appkit|xcode|clickhouse|django|spring|kotlin|java|cpp|android|ios|electron|tauri|golang|git|github|database|postgres|sql|api|webapp)\b|开发|编程|调试|测试|数据库/],
  ['tools', /\b(search|browser|calendar|weather|email|files|file management|productivity|time|task management|brainstorming|planning)\b|搜索|浏览器|日历|天气|邮件|文件管理/],
];
const sourceCategories = new Map(sources.filter(s => s.category).map(s => [s.value.toLowerCase(), s.category]));

export function classifySkill(skill, repo = skill.source?.repo) {
  // Only use the immediate directory; parent plugin/vendor names can be misleading.
  const directory = (skill.directory || skill.source?.directory || skill.path || '').replace(/\\/g, '/').split('/').filter(Boolean).at(-1) || '';
  const text = `${skill.name || ''} ${directory}`.toLowerCase().replace(/[-_./]+/g, ' ');
  return rules.find(([, pattern]) => pattern.test(text))?.[0]
    || sourceCategories.get((repo || '').toLowerCase()) || 'uncategorized';
}

export function skillCategoryName(id) {
  return skillCategories.find(category => category.id === id)?.name || '未分类';
}

export function skillCategoryCounts(items) {
  const counts = Object.fromEntries(skillCategories.map(category => [category.id, 0]));
  counts[''] = items.length;
  for (const item of items) counts[item.category]++;
  return counts;
}
