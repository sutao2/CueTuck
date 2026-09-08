// Explicit local development import; no production URL or destructive mode.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../', import.meta.url));
const destination = `${root}samples/community/prompts.json`;
const hash = text => createHash('sha256').update(text).digest('hex');
const selectedZh = [
  ['充当前端智能思路助手','cat-software-1'], ['充当正则表达式生成器','cat-software-3'],
  ['作为专业DBA','cat-software-2'], ['担任网页设计顾问','cat-software-0'],
  ['充当 Excel 工作表','cat-office-1'], ['充当英语翻译和改进者','cat-office-2'],
  ['充当讲故事的人','cat-writing-1'], ['充当小说家','cat-writing-1'],
  ['充当诗人','cat-writing-1'], ['充当花哨的标题生成器','cat-writing-0'],
  ['作为 UX/UI 开发人员','cat-product-2'], ['作为广告商','cat-marketing-0'],
  ['担任统计员','cat-data-1'], ['充当 SQL 终端','cat-data-0'],
  ['担任数学老师','cat-education-1'], ['担任哲学老师','cat-education-0'],
  ['担任 AI 写作导师','cat-education-1'], ['担任辩论教练','cat-education-0'],
  ['充当旅游指南','cat-life-0'], ['担任面试官','cat-life-2'],
  ['担任职业顾问','cat-life-2'], ['担任厨师','cat-life-1'],
  ['担任编剧','cat-video-0'], ['担任作曲家','cat-video-0'],
];
const selectedEn = [
  ['Software Quality Assurance Tester','软件质量测试员','cat-software-3'],
  ['Fullstack Software Developer','全栈软件开发','cat-software-0'],
  ['Machine Learning Engineer','机器学习工程师','cat-software-2'],
  ['Product Manager','产品经理 · PRD','cat-product-0'],
  ['Data Scientist','数据科学家','cat-data-1'], ['Data Analyst','数据分析师','cat-data-2'],
  ['Social Media Manager','社交媒体运营','cat-marketing-1'],
  ['Social Media Influencer','社交媒体内容策划','cat-writing-0'],
  ['Excel Sheet','Excel 表格助手','cat-office-1'],
  ['UX/UI Developer','UX/UI 体验设计','cat-product-2'],
  ['Travel Guide','旅行向导','cat-life-0'], ['Screenwriter','电影剧本写作','cat-video-0'],
];
// CSV supports quoted newlines and escaped quotes used by prompts.chat.
export function csv(text) {
  const rows = []; let row = [], field = '', quoted = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '"') { if (quoted && text[i + 1] === '"') { field += '"'; i++; } else quoted = !quoted; }
    else if (c === ',' && !quoted) { row.push(field); field = ''; }
    else if (c === '\n' && !quoted) { row.push(field.replace(/\r$/, '')); rows.push(row); row = []; field = ''; }
    else field += c;
  }
  if (quoted) throw Error('Unclosed CSV quote');
  if (field || row.length) rows.push([...row, field]);
  const headers = rows.shift();
  return rows.map(values => Object.fromEntries(headers.map((key, i) => [key, values[i]])));
}
function build(sourceDir) {
  const read = name => readFileSync(`${sourceDir}/${name}`, 'utf8');
  const records = [];
  function add(title, content, category_id, model, reference, excerpt) {
    if (!content?.trim() || !reference.url) throw Error(`Incomplete source: ${title}`);
    const notice = `来源：${reference.repository}\n作者：${reference.author}\n原文：${reference.url}\n许可：${reference.license} ${reference.license_url}\n修改：正文保留原文，另加中文展示标题、分类与来源说明；仅本机学习预览。`;
    records.push({ id: `community-${hash(reference.repository + ':' + reference.url + ':' + title).slice(0, 20)}`, title, kind: 'prompt', content: `${content.trim()}\n\n---\n${notice}`, excerpt: excerpt || content.trim().slice(0, 120), category_id, model, reference });
  }
  const youmind = read('youmind.md').split(/^### No\. \d+: /m).slice(1);
  // Seven editorial/product examples, then five portrait/illustration examples.
  const imageTitles = [
    '带肖像和中英文定制的宽引言卡','高级液态玻璃 Bento 网格产品信息图，含 8 个模块',
    '手绘风格标题图片提示（来自照片）','德国水彩地图，附带各州名称','元旦特辑：2026 祝福四格拼图',
    '一项发明的复古专利文件','江户时代浮世绘风格的现代场景再现',
    '个人资料 / 头像 - 印度女性窗边肖像','个人资料 / 头像 - 电影感强烈侧视肖像',
    '个人资料 / 头像 - 自然美感影棚人像','个人资料 / 头像 - 素描风格肖像插画',
    '个人资料 / 头像 - 动漫艺术风格月光山脉',
  ];
  for (const title of imageTitles) {
    const block = youmind.find(b => b.split('\n')[0].trim() === title);
    if (!block) throw Error(`Missing YouMind title: ${title}`);
    const content = block.match(/#### 📝 提示词\s+```[^\n]*\n([\s\S]*?)\n```/)?.[1];
    const author = block.match(/\*\*作者:\*\* \[([^\]]+)\]/)?.[1];
    const url = block.match(/https:\/\/youmind.com\/zh-CN\/nano-banana-pro-prompts\?id=\d+/)?.[0];
    const images = [...block.matchAll(/<img src="(https:\/\/cms-assets.youmind.com\/[^"\s]+)"/g)].map(m => m[1]).slice(0, 4);
    if (!author || !images.length) throw Error(`Missing image attribution: ${title}`);
    const category = title.includes('产品信息图') ? 'cat-image-1' : /肖像|人像/.test(title) && !/卡|插画/.test(title) ? 'cat-image-0' : 'cat-image-2';
    add(title, content, category, 'Nano Banana', { repository:'YouMind-OpenLab/awesome-nano-banana-pro-prompts', author, url, license:'CC BY 4.0', license_url:'https://creativecommons.org/licenses/by/4.0/', images, source_model:'Nano Banana Pro' }, block.match(/#### 📖 描述\s+([\s\S]*?)\n\n####/)?.[1]);
  }
  const zh = JSON.parse(read('zh.json'));
  for (const [title, category] of selectedZh) {
    const row = zh.find(r => r.act === title); if (!row) throw Error(`Missing: ${title}`);
    add(title, row.prompt + '\n\n' + read('zh-license.txt'), category, 'ChatGPT', { repository:'PlexPt/awesome-chatgpt-prompts-zh', author:'PlexPt 与社区贡献者', url:'https://github.com/PlexPt/awesome-chatgpt-prompts-zh/blob/main/prompts-zh.json', license:'MIT', license_url:'https://github.com/PlexPt/awesome-chatgpt-prompts-zh/blob/main/LICENSE', images:[] }, row.prompt.trim().slice(0,120));
  }
  const en = csv(read('f.csv'));
  for (const [act, title, category] of selectedEn) {
    const row = en.find(r => r.act === act); if (!row) throw Error(`Missing: ${act}`);
    add(`${title}（英文）`, row.prompt, category, 'ChatGPT', { repository:'f/prompts.chat', author:row.contributor || 'prompts.chat contributors', url:'https://github.com/f/prompts.chat/blob/main/prompts.csv', license:'CC0 1.0', license_url:'https://creativecommons.org/publicdomain/zero/1.0/', images:[], original_title:act });
  }
  const manifest = { imported_at: new Date().toISOString(), purpose:'local-development-preview', records };
  validate(manifest);
  mkdirSync(`${root}samples/community`, {recursive:true});
  writeFileSync(destination, JSON.stringify(manifest, null, 2) + '\n');
  for (const name of ['youmind-license.txt','zh-license.txt','cc0.txt']) writeFileSync(`${root}samples/community/${name}`, read(name));
  console.log(`Prepared ${records.length} records`);
}
export function validate(manifest) {
  if (manifest.purpose !== 'local-development-preview' || manifest.records.length !== 48) throw Error('Expected 48 reviewed local samples');
  const ids = new Set(), contents = new Set();
  for (const r of manifest.records) {
    if (!/^community-[a-f0-9]{20}$/.test(r.id) || ids.has(r.id) || contents.has(r.content) || !r.content || !r.reference.license || !r.reference.author || !r.content.includes(r.reference.license_url)) throw Error(`Invalid or duplicate record: ${r.id}`);
    for (const image of r.reference.images) if (!/^https:\/\/cms-assets\.youmind\.com\//.test(image)) throw Error('Unreviewed image host');
    ids.add(r.id); contents.add(r.content);
  }
}
export function importSql(manifest) {
  validate(manifest);
  const data = JSON.stringify(manifest.records).replaceAll("'", "''");
  return `BEGIN;
SELECT pg_advisory_xact_lock(80480908);
CREATE TEMP TABLE incoming AS SELECT * FROM jsonb_to_recordset('${data}'::jsonb) AS x(id text,title text,kind text,content text,excerpt text,category_id text,model text,reference jsonb);
DO $$ BEGIN IF EXISTS(SELECT 1 FROM incoming i WHERE NOT EXISTS(SELECT 1 FROM public.catalog c WHERE c.kind='categories' AND c.id=i.category_id AND (c.data->>'enabled')::boolean AND NOT c.deleted) OR NOT EXISTS(SELECT 1 FROM public.catalog c WHERE c.kind='models' AND c.id=i.model AND (c.data->>'enabled')::boolean AND NOT c.deleted)) THEN RAISE EXCEPTION 'Missing enabled catalog reference'; END IF; END $$;
WITH added AS (INSERT INTO public.square_items(id,title,kind,content,excerpt,category_id,model,reference,sort_index) SELECT id,title,kind,content,excerpt,category_id,model,reference,100 FROM incoming ON CONFLICT(id) DO NOTHING RETURNING id) SELECT count(*) AS inserted FROM added;
COMMIT;`;
}
function apply() {
  const manifest = JSON.parse(readFileSync(destination, 'utf8'));
  const sql = importSql(manifest);
  const run = (args, input) => { const r = spawnSync('docker', args, {input, maxBuffer:32*1024*1024}); if (r.status !== 0) throw Error(r.stderr.toString()); return r.stdout; };
  const context = JSON.parse(run(['context','inspect']).toString())[0];
  if (!context.Endpoints.docker.Host.startsWith('unix://')) throw Error('Only local Docker is allowed');
  const backup = `${root}output/community-backup-${Date.now()}.sql`;
  mkdirSync(`${root}output`, {recursive:true});
  writeFileSync(backup, run(['exec','backend-db-1','pg_dump','-U','pl','-d','promptark','--table=public.square_items']), {mode:0o600});
  console.log(`Square table backup: ${backup}`);
  console.log(run(['exec','-i','backend-db-1','psql','-X','-v','ON_ERROR_STOP=1','-U','pl','-d','promptark'],sql).toString());
}
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === '--build' && process.argv[3]) build(process.argv[3]);
  else if (process.argv[2] === '--apply-local') apply();
  else { const manifest = JSON.parse(readFileSync(destination, 'utf8')); validate(manifest); console.log(`Dry run: ${manifest.records.length} records; use --apply-local for localhost Docker only.`); }
}
