import { highlightedParts } from './shared/search.mjs';
import { PAGE_SIZE, requestJson, browsePrompts, categoryCount, safeImage, safeLink, skillCatalog, readSkill, skillDescription } from './live-data.mjs';
import { classifySkill, skillCategories } from './shared/skillCategories.mjs';
import { extractVariables, variableDefaults, renderPrompt } from './shared/renderPrompt.mjs';
const $ = (s, root=document) => root.querySelector(s);
const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const highlighted = text => highlightedParts(text,query).map(part=>part.match?`<mark>${esc(part.text)}</mark>`:esc(part.text)).join('');
const fmt = value => Number(value || 0).toLocaleString('zh-CN');
const API = ''; // Public browsing uses the existing same-origin API.
const paths={search:'<circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 4 4"/>',grid:'<rect x="3" y="3" width="6" height="6" rx="1"/><rect x="14" y="3" width="6" height="6" rx="1"/><rect x="3" y="14" width="6" height="6" rx="1"/><rect x="14" y="14" width="6" height="6" rx="1"/>',list:'<path d="M8 5h13M8 12h13M8 19h13M3 5h.1M3 12h.1M3 19h.1"/>',star:'<path d="m12 3 2.8 5.7 6.2.9-4.5 4.4 1.1 6.2-5.6-2.9-5.6 2.9 1.1-6.2L3 9.6l6.2-.9Z"/>',arrow:'<path d="M5 12h14m-5-5 5 5-5 5"/>',code:'<path d="m8 6-6 6 6 6m8-12 6 6-6 6M14 3l-4 18"/>',write:'<path d="m4 16-1 5 5-1L21 7l-4-4Zm10-10 4 4"/>',image:'<rect x="3" y="3" width="18" height="18" rx="3"/><circle cx="9" cy="8" r="2"/><path d="m3 17 5-5 4 4 4-6 5 7"/>',brief:'<rect x="3" y="6" width="18" height="15" rx="2"/><path d="M8 6V3h8v3M3 12h18m-11 0v3h4v-3"/>',folder:'<path d="M3 7V4h6l3 3h9v13H3Z"/>',spark:'<path d="m12 2 3 7 7 3-7 3-3 7-3-7-7-3 7-3Z"/>',plus:'<path d="M12 5v14M5 12h14"/>',copy:'<rect x="8" y="8" width="12" height="13" rx="2"/><path d="M16 8V3H3v13h5"/>'};
const icon = n => `<svg aria-hidden="true" viewBox="0 0 24 24">${paths[n]||paths.folder}</svg>`;
const saved = new Map(), drafts = [], repoCache = new Map(), bodyCache = new Map();
let recommendation=null, recommendationFilter='', recentRecommendations=[], recommendationExclude=[], dismissed=[];
try { const stored=JSON.parse(localStorage.getItem('cuetuck-dismissed') || '[]'); if(Array.isArray(stored))dismissed=stored.filter(id=>typeof id==='string'&&id.length<=200).slice(0,200); } catch {}
function saveDismissed(ids) { try { localStorage.setItem('cuetuck-dismissed',JSON.stringify(ids)); dismissed=ids; return true; } catch { notify('偏好保存失败，请检查浏览器存储权限。'); return false; } }
let route='', category='', query='', model='', sort='推荐', language='zh', view='grid', offset=0;
let catalog=null, sources=null, repo='anthropics/skills', skillData=null, items=[], total=0, nextOffset=null, counts=null, allCount=null;
let busy=false, listError='', generation=0, controller, searchTimer, toastTimer, detailController, detailGeneration=0, activeItem, activeBody='';
function notify(message) { $('.toast').textContent=message; $('.toast').hidden=false; clearTimeout(toastTimer); toastTimer=setTimeout(()=>$('.toast').hidden=true,3500); }
function sourceURL(item) { return `https://github.com/${item.repo}/blob/${item.commit}/${item.path.split('/').map(encodeURIComponent).join('/')}`; }
function home() { return `<div class="landing">
<section class="launch-hero">
  <div class="launch-copy"><a class="release-pill" href="https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.18" target="_blank" rel="noopener"><span></span> v0.1.0-beta.18 · 现已发布 ${icon('arrow')}</a><p class="launch-eyebrow">THE PROMPT LAUNCHER</p><h1>好提示词，<br><em>一键就位。</em></h1><p class="launch-intro">CueTuck · 唤词，你的桌面提示词启动器。<br>一个快捷键，找到模板、填好变量、回车复制。<br>从正在做的事，直接接上下一步。</p><div class="launch-actions"><a class="button dark" href="#download">下载 CueTuck ${icon('arrow')}</a><a class="launch-secondary" href="#app">体验网页广场 ↗</a></div><p class="launch-meta">macOS & Windows <span>·</span> 本地优先 <span>·</span> 开源</p></div>
  <div class="launcher-stage"><div class="stage-top"><span><img src="assets/icon.png" alt=""> 随时唤起，即刻使用</span><span class="stage-shortcut"><kbd>⌃</kbd><kbd>Space</kbd></span></div><div class="launcher-screen"><img id="launcher-preview" src="assets/launcher-search.png" width="760" height="560" alt="CueTuck 独立启动器：搜索产品方案提示词，使用键盘选择结果" fetchpriority="high"></div><div class="launcher-switch" role="group" aria-label="启动器界面预览"><button data-launcher-preview="search" aria-pressed="true">01 搜索提示词</button><button data-launcher-preview="variables" aria-pressed="false">02 填写与复制</button></div><p class="stage-caption">实际应用界面 · 示例内容 · 全局唤起需桌面版</p></div>
</section>
<section class="key-flow" aria-label="启动器使用流程"><div><span class="step-number">01</span><p><strong>随时唤起</strong><span>默认 Control + Space，可在设置中修改</span></p><kbd>⌃ Space</kbd></div><div><span class="step-number">02</span><p><strong>输入，找到</strong><span>搜索本地提示词，用方向键选择</span></p><kbd>↑ ↓</kbd></div><div><span class="step-number">03</span><p><strong>填写，复制</strong><span>有变量先填写，确认结果后复制使用</span></p><kbd>↵</kbd></div></section>
<section class="launcher-depth"><div class="depth-heading"><p class="launch-eyebrow">STAY IN YOUR FLOW</p><h2>思路不断，<br>接着往下做。</h2><p>写代码、做内容、整理资料。<br>常用的 AI 工作方式，都从同一个入口开始。</p></div><div class="depth-features"><article><span>${icon('search')}</span><div><h3>找回那句好用的提示词</h3><p>搜索自己的本地库，选中结果直接使用。不用反复翻聊天记录，或重新写一遍指令。</p></div></article><article><span>${icon('write')}</span><div><h3>新想法，顺手收好</h3><p>直接在启动器输入内容，选择创建提示词。下次需要，它就在你的本地库里。</p></div></article><article><span>${icon('spark')}</span><div><h3>让 AI 帮你把话说清楚</h3><p>选择 AI 优化，使用你在客户端配置的模型改写输入；也可以主动搜索广场，找一个更好的起点。</p></div></article></div></section>
<section class="collection-section"><div class="collection-heading"><div><p class="launch-eyebrow">COLLECT ONCE. USE ANYTIME.</p><h2>启动器背后，<br>是你的提示词收藏库。</h2></div><p>分类、变量、参考图片，都整理在一起。<br>从广场下载好内容，变成自己随时可用的模板。</p></div><div class="collection-shot"><img src="assets/library.png" alt="CueTuck 本地提示词库：分类、搜索与可复用的提示词" width="1440" height="960" loading="lazy"></div><div class="collection-links"><a href="#app"><span>${icon('grid')} 提示词广场</span><span>按用途和模型发现社区内容 ↗</span></a><a href="#skills"><span>${icon('code')} Skills 也有自己的位置</span><span>网页查看来源，桌面创建、发布与安装 ↗</span></a></div></section>
<section class="trust-strip"><div><strong>本地优先</strong><p>自己的提示词存入本机 SQLite，日常整理不依赖账号。</p></div><div><strong>模型由你选择</strong><p>优化与本地翻译使用你配置的服务，密钥保存在系统凭据库。</p></div><div><strong>与你的工具配合</strong><p>复制到常用 AI 软件，也可通过 MCP 让智能体读取本地提示词。</p></div></section>
<section class="launch-end"><img src="assets/icon.png" alt="" width="48" height="48"><h2>下一句好提示词，<br>只隔一个快捷键。</h2><p>Your prompts, a shortcut away.</p><a class="button dark" href="#download">下载 CueTuck · 唤词 ${icon('arrow')}</a><a class="launch-secondary" href="https://github.com/sutao2/CueTuck" target="_blank" rel="noopener">在 GitHub 查看源码 ↗</a></section>
</div>`; }
function downloads() { return `<section class="section download-section"><span class="eyebrow">DOWNLOAD CUETUCK</span><h1>下载桌面客户端</h1><p class="muted">本机库、全局启动器与 Skills 管理。当前预览版 v0.1.0-beta.18。</p><div class="download-grid">${[['macOS','Apple 芯片 · DMG','aarch64.dmg'],['Windows','64 位 · EXE','x64-setup.exe']].map(([name,description,file])=>`<article class="download-card"><h2>${name}</h2><p>${description}</p><a class="button dark" href="https://github.com/sutao2/CueTuck/releases/download/v0.1.0-beta.18/CueTuck_0.1.0-beta.18_${file}">下载 ${name} 版 ↓</a></article>`).join('')}</div><p class="download-note">预览版未完成 Apple 公证或 Windows 发布者签名，首次打开可能出现系统提示。</p><a class="text-link" href="https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.18" target="_blank" rel="noopener">发行说明与 SHA-256 校验文件 ↗</a></section>`; }
function nameOf(id) { return catalog?.categories.find(c=>c.id===id)?.name || skillCategories.find(c=>c.id===id)?.name || '未分类'; }
function categoryRows() {
  if (route==='skills') return skillCategories.map(c=>({...c,count:skillData ? (c.id ? skillData.entries.filter(e=>e.category_id===c.id).length : skillData.entries.length) : null}));
  if (route!=='app') return [];
  const ordered=(catalog?.categories || []).filter(c=>c.enabled).sort((a,b)=>a.sort_index-b.sort_index);
  const selectedParent=ordered.find(c=>c.id===category)?.parent_id || category;
  const roots=ordered.filter(c=>!c.parent_id);
  return [{id:'',name:'全部提示词',count:allCount}, ...roots.flatMap(c=>[
    {...c,count:categoryCount(c.id,ordered,counts)},
    ...(selectedParent===c.id?ordered.filter(child=>child.parent_id===c.id).map(child=>({...child,child:true,count:counts?.[child.id] ?? (counts?0:null)})):[])
  ])];
}
function renderCategories() {
  const html=categoryRows().map(c=>`<button class="side-link ${c.id===category?'active':''} ${c.child?'child':''}" data-category="${esc(c.id)}" aria-pressed="${c.id===category}">${c.child?'':icon('folder')}<span>${esc(c.name)}</span><span class="count">${c.count===null?'—':fmt(c.count)}</span></button>`).join('');
  if ($('#categories')) $('#categories').innerHTML=html;
  if ($('#mobile-category')) $('#mobile-category').innerHTML=`<option value="">全部分类</option>${categoryRows().filter(c=>c.id).map(c=>`<option value="${esc(c.id)}" ${c.id===category?'selected':''}>${esc(c.name)}</option>`).join('')}`;
}
function workspace() {
  const titles={app:['提示词广场','浏览社区发布的提示词，按用途与模型查找。'],skills:['Skill 广场','浏览与客户端相同的公开 GitHub 来源。'],favorites:['本页暂存','当前标签页暂存的内容，刷新或关闭后清空。'],drafts:['本页草稿','仅保留在当前标签页，不写入账号库或桌面本机库。']};
  const [title,subtitle]=titles[route];
  return `<section class="workspace"><aside class="sidebar"><span class="sidebar-heading">浏览</span><a class="side-link ${route==='app'?'active':''}" href="#app">${icon('grid')}提示词广场</a><a class="side-link ${route==='skills'?'active':''}" href="#skills">${icon('code')}Skill 广场</a><div class="side-separator"></div><span class="sidebar-heading">本页内容</span><a class="side-link ${route==='favorites'?'active':''}" href="#favorites">${icon('star')}本页暂存<span class="count" data-saved-count>${saved.size}</span></a><a class="side-link ${route==='drafts'?'active':''}" href="#drafts">${icon('write')}本页草稿<span class="count">${drafts.length}</span></a><div class="side-separator"></div><span class="sidebar-heading">分类</span><div id="categories"></div><div class="side-bottom"><a class="side-link" href="#download">${icon('folder')}下载桌面客户端 ↗</a><button class="side-link" data-account>${icon('brief')}账号与数据说明</button></div></aside><div class="work-content"><div class="breadcrumb">CueTuck / ${title}</div><div class="work-heading"><div><h1>${title}</h1><p>${subtitle}</p></div><button class="button" data-create>${icon('plus')}新建草稿</button></div>${route==='skills'?`<div class="source-panel"><label for="repo">公开来源</label><input id="repo" list="sources" value="${esc(repo)}" aria-label="搜索或选择 Skill 来源"><datalist id="sources"></datalist><button class="button small" data-refresh>刷新来源</button><p id="source-meta">正在读取公开目录…</p></div>`:''}<div class="mobile-navigation"><a href="#favorites">本页暂存</a><a href="#drafts">草稿</a><select id="mobile-category" aria-label="选择分类"></select></div><div class="toolbar"><label class="search-field">${icon('search')}<input type="search" id="search" aria-label="搜索内容" placeholder="${route==='skills'?'搜索当前来源的名称、路径或已加载说明':'搜索标题或正文…'}" value="${esc(query)}"></label>${route==='app'?`<input id="model" list="models" aria-label="搜索或选择模型" placeholder="全部模型" value="${esc(model)}"><datalist id="models"></datalist><select id="language" aria-label="正文语言"><option value="zh" ${language==='zh'?'selected':''}>中文优先</option><option value="original" ${language==='original'?'selected':''}>原文</option></select>`:''}<div class="view-controls"><button data-view="grid" class="${view==='grid'?'active':''}" aria-label="网格视图">${icon('grid')}</button><button data-view="list" class="${view==='list'?'active':''}" aria-label="列表视图">${icon('list')}</button></div></div><div class="results-heading"><div class="result-tabs">${route==='app'?['推荐','最新','热门'].map(s=>`<button data-sort="${s}" class="${sort===s?'active':''}">${s}</button>`).join(''):'<span>当前结果</span>'}</div><span id="result-count" aria-live="polite"></span>${route==='app'?'<button class="button small" data-refresh id="ranking-refresh">换一批</button><button class="button small" data-restore-recommendations>恢复推荐偏好</button>':''}</div><p id="ranking-note" class="source-meta"></p><div id="cards" class="cards ${view==='list'?'list':''}"></div><div id="pagination"></div></div></section>`;
}
function fillDictionaries() {
  renderCategories();
  if ($('#models')) $('#models').innerHTML=(catalog?.models || []).filter(m=>m.enabled).map(m=>`<option value="${esc(m.id)}">${esc(m.name)}</option>`).join('');
  if ($('#sources')) $('#sources').innerHTML=(sources || []).map(s=>`<option value="${esc(s.value)}">${esc(s.label)}</option>`).join('');
  if ($('#source-meta')) $('#source-meta').textContent=skillData ? `${skillData.repo} · 提交 ${skillData.commit.slice(0,12)} · ${fmt(skillData.entries.length)} 个 Skills · ${sources?.length || 0} 个可选来源` : '选择来源后读取真实目录。';
}
function renderCards() {
  if (!$('#cards')) return;
  $('#result-count').textContent=busy?'正在加载…':listError?'加载失败':`共 ${fmt(total)} 个结果`;
  if (listError) $('#cards').innerHTML=`<div class="empty" role="alert"><h2>暂时无法加载内容</h2><p>${esc(listError)}</p><button class="button" data-refresh>重试</button></div>`;
  else if (busy) $('#cards').innerHTML='<div class="loading" role="status">正在读取内容…</div>';
  else if (!items.length) $('#cards').innerHTML=`<div class="empty"><h2>${route==='favorites'?'还没有暂存内容':route==='drafts'?'还没有本页草稿':'没有匹配的内容'}</h2><p>${route==='favorites'?'点击条目星标，暂存到当前标签页。':'可以清除筛选，或尝试其他关键词。'}</p><button class="button" data-clear>清除筛选</button></div>`;
  else $('#cards').innerHTML=items.map(item=>{
    const image=safeImage(item,API), isSkill=item.kind==='skill';
    const subtitle=isSkill?item.repo:item.publisher?.display_name || item.reference?.author || (item.kind==='draft'?'本页草稿':'公开内容');
    return `<article class="prompt-card ${image?'with-image':''} ${isSkill?'skill-card':''}">${image?`<button class="card-cover" data-open="${esc(item.id)}" tabindex="-1" aria-label="查看 ${esc(item.title)}"><img src="${esc(image)}" alt="${esc(item.title)}" loading="lazy" decoding="async" referrerpolicy="no-referrer"></button>`:''}<div class="card-main"><div class="card-top"><span class="card-model">${esc(item.model || (isSkill?'Skill':'通用'))}</span><span>${esc(nameOf(item.category_id))}</span></div><button class="card-title" data-open="${esc(item.id)}">${highlighted(item.title)}</button><p class="card-description">${highlighted(item.excerpt || item.description || (isSkill?'打开查看 SKILL.md 内容与来源说明。':item.content || '打开查看正文'))}</p>${isSkill?`<code class="source-path">${esc(item.directory || '仓库根目录')}</code>`:''}<div class="card-bottom"><span class="author" title="${esc(subtitle)}">${esc(subtitle)}</span><div class="card-actions">${route==='app'&&sort==='推荐'?`<button class="icon-button" data-dismiss="${esc(item.id)}" aria-label="不感兴趣：${esc(item.title)}" title="不感兴趣">×</button>`:''}<button class="icon-button ${saved.has(item.id)?'saved':''}" data-save="${esc(item.id)}" title="${saved.has(item.id)?'取消暂存':'暂存到本页，刷新后清空'}" aria-label="${saved.has(item.id)?'取消暂存':'暂存'} ${esc(item.title)}" aria-pressed="${saved.has(item.id)}">${icon('star')}</button><button class="icon-button" data-open="${esc(item.id)}" aria-label="查看 ${esc(item.title)}">${icon('arrow')}</button></div></div>${!isSkill && item.kind!=='draft'?`<div class="metrics"><span>${fmt(item.download_count)} 下载</span><span>${fmt(item.favorite_count)} 收藏</span></div>`:''}</div></article>`;
  }).join('');
  $('#pagination').innerHTML=`<span>${busy?'':items.length?`第 ${Math.floor(offset/PAGE_SIZE)+1} 页 · 本页 ${items.length} 条`:''}</span><div><button class="button small" data-page="prev" ${busy||offset===0?'disabled':''}>上一页</button><button class="button small" data-page="next" ${busy||nextOffset===null?'disabled':''}>下一页</button></div>`;
  document.querySelectorAll('[data-saved-count]').forEach(el=>el.textContent=saved.size);
}
async function loadList() {
  clearTimeout(searchTimer); controller?.abort(); controller=new AbortController(); const signal=controller.signal, current=++generation;
  busy=true; listError=''; items=[]; renderCards();
  try {
    if (route==='app') {
      $('#ranking-refresh').textContent=sort==='推荐'?'换一批':'刷新';
      $('#ranking-note').textContent=sort==='推荐'?'精选、下载热度与新内容 · 每小时轮换，翻页保持本轮顺序':sort==='最新'?'按上架时间从新到旧排序':'按已记录累计下载量从高到低排序';
      if (!catalog) catalog=await requestJson(`${API}/v1/square/catalog`,{signal});
      if (current!==generation) return;
      fillDictionaries();
      const filter=JSON.stringify([query,category,model,sort,language]);
      if(filter!==recommendationFilter){recommendation=null;recommendationFilter=filter;}
      if(offset===0)recommendationExclude=sort==='推荐'?[...new Set([...dismissed,...recentRecommendations])]:[];
      const page=await browsePrompts(API,{query,category,model,sort,language,offset,recommendation,exclude:recommendationExclude},{signal});
      if (current!==generation) return;
      recommendation=page.recommendation || recommendation; items=page.items; total=page.total; nextOffset=page.next_offset;
      if (page.category_counts) { counts=page.category_counts; allCount=page.category_total; }
    } else {
      let rows;
      if (route==='skills') {
        if (!sources) sources=await requestJson('./data/skill-sources.json',{signal});
        fillDictionaries();
        if (!repoCache.has(repo)) repoCache.set(repo,await skillCatalog(repo,{signal}));
        if (current!==generation) return;
        skillData=repoCache.get(repo);
        skillData.entries.forEach(e=>e.category_id=classifySkill({name:e.title,directory:e.directory},e.repo));
        rows=skillData.entries;
      } else rows=route==='favorites'?[...saved.values()]:drafts;
      rows=rows.filter(e=>(!category||e.category_id===category)&&`${e.title} ${e.directory||''} ${e.description||''} ${e.content||''}`.toLowerCase().includes(query.toLowerCase()));
      total=rows.length; items=rows.slice(offset,offset+PAGE_SIZE); nextOffset=offset+PAGE_SIZE<total?offset+PAGE_SIZE:null;
    }
  } catch(error) {
    if (current!==generation) return;
    listError=error.name==='TimeoutError'?'请求超时，请重试。':error.message || '加载失败，请重试。';
  } finally {
    if (current===generation) { busy=false; if(route==='app')$('#ranking-refresh').textContent=sort==='推荐'?(items.length||!recentRecommendations.length?'换一批':'重新浏览'):'刷新'; fillDictionaries(); renderCards(); if (route==='skills'&&!listError) loadDescriptions(current,signal); }
  }
}
async function loadDescriptions(current,signal) {
  const queue=items.filter(item=>!item.description && !item.descriptionTried);
  await Promise.all(Array.from({length:Math.min(4,queue.length)},async()=>{
    while(queue.length && !signal.aborted && current===generation) {
      const item=queue.shift(); item.descriptionTried=true;
      try { const text=bodyCache.get(item.id+item.commit) || await readSkill(item,{signal}); bodyCache.set(item.id+item.commit,text); item.description=skillDescription(text); }
      catch { if(signal.aborted){item.descriptionTried=false;return;} item.description='说明未读取，可在详情中重试或访问来源。'; }
      if (current===generation) {
        const card=[...document.querySelectorAll('[data-open]')].find(el=>el.dataset.open===item.id)?.closest('.prompt-card');
        if (card) $('.card-description',card).textContent=item.description || '打开查看 SKILL.md。';
      }
    }
  }));
}
function resetFilters() { category=''; query=''; model=''; offset=0; recommendation=null; recommendationFilter=''; }
function render() {
  clearTimeout(searchTimer);
  controller?.abort(); generation++; detailController?.abort(); detailGeneration++;
  document.querySelectorAll('dialog[open]').forEach(d=>d.close());
  const next=location.hash.slice(1)||'home'; route=['home','app','skills','favorites','drafts','download'].includes(next)?next:'home';
  resetFilters(); $('#main').innerHTML=route==='home'?home():route==='download'?downloads():workspace();
  $('#footer').hidden=!['home','download'].includes(route);
  document.querySelectorAll('[data-nav]').forEach(a=>{ a.classList.toggle('active',a.dataset.nav===route); if(a.dataset.nav===route)a.setAttribute('aria-current','page');else a.removeAttribute('aria-current'); });
  $('.nav-end .button').href='#download'; $('.nav-end .button').textContent='下载 CueTuck ↗';
  document.title=`${{home:'提示词启动器，一键唤起灵感',app:'提示词广场',skills:'Skill 广场',favorites:'本页暂存',drafts:'本页草稿',download:'下载'}[route]} · CueTuck 唤词`;
  if (!['home','download'].includes(route)) { fillDictionaries(); loadList(); }
  window.scrollTo(0,0);
}
function currentItem(id) { return items.find(e=>e.id===id) || saved.get(id) || drafts.find(e=>e.id===id); }
async function openItem(id) {
  const item=currentItem(id); if (!item) return;
  activeItem=item; detailController?.abort(); detailController=new AbortController();const signal=detailController.signal, current=++detailGeneration;
  $('#detail-body').innerHTML=`<h2 id="dialog-title">${esc(item.title)}</h2><p role="status">正在加载完整内容…</p>`; if(!$('#detail').open)$('#detail').showModal();
  try {
    let result=item, body='';
    if (item.kind==='skill') {
      body=bodyCache.get(item.id+item.commit) || await readSkill(item,{signal}); bodyCache.set(item.id+item.commit,body);
    } else if (item.kind==='draft') body=item.content;
    else {
      const data=await requestJson(`${API}/v1/square/items/${encodeURIComponent(item.id)}/content`,{signal});
      if (current!==detailGeneration) return;
      result={...item,...data};
      const translated=data.translations?.[language]?.version;
      body=translated?.content ?? data.content ?? '';
      if (data.kind==='collection') body=(translated?.members || data.members || []).map(m=>`${m.title}\n${m.content || ''}`).join('\n\n');
    }
    if (current!==detailGeneration) return;
    activeItem=result; activeBody=body; renderDetail();
  } catch(error) {
    if(current===detailGeneration && !signal.aborted)$('#detail-body').innerHTML=`<h2 id="dialog-title">${esc(item.title)}</h2><p role="alert">${esc(error.message)}</p><button class="button" data-retry-detail>重试</button>`;
  }
}
function renderDetail() {
  const p=activeItem, skill=p.kind==='skill', vars=skill?[]:extractVariables(activeBody), defaults=variableDefaults(activeBody), image=safeImage(p,API);
  const url=skill?sourceURL(p):safeLink(p.reference?.url);
  $('#detail-body').innerHTML=`<h2 id="dialog-title">${esc(p.title)}</h2><div class="detail-meta"><span>${esc(skill?p.repo:p.publisher?.display_name||p.reference?.author||'未提供作者信息')}</span><span>${esc(p.model || '')}</span>${url?`<a class="text-link" href="${esc(url)}" target="_blank" rel="noopener">查看来源 ↗</a>`:''}</div>${skill?`<p class="source-path">提交 ${p.commit.slice(0,12)} · ${esc(p.directory || '仓库根目录')}</p>`:''}${image?`<button class="detail-image" data-image="${esc(image)}" aria-label="放大参考图片"><img src="${esc(image)}" alt="${esc(p.title)}" referrerpolicy="no-referrer"></button>`:''}${p.reference?.license?`<p class="license">来源许可：${esc(p.reference.license)}</p>`:''}<div class="variable-fields">${vars.map(name=>`<label>${esc(name)}<input data-variable="${esc(name)}" value="${esc(defaults[name] || '')}" placeholder="填写${esc(name)}"></label>`).join('')}</div><div class="prompt-text ${skill?'code-document':''}" id="prompt-text"></div><div class="dialog-actions"><button class="button" data-save="${esc(p.id)}" data-detail-save>${saved.has(p.id)?'取消暂存':'暂存到本页'}</button>${skill?'<a class="button dark" href="#download" data-close>使用桌面版安装 ↗</a>':`<button class="button dark" data-copy>${icon('copy')}复制提示词</button>`}</div><p class="detail-note">${skill?'真实 SKILL.md 原文。网页只读展示，不执行文件中的指令；依赖与许可请核对来源。':'填写与复制在当前页面处理；暂存不等于账号收藏，刷新后清空。'}</p>`;
  updatePreview();
}
function updatePreview() { if($('#prompt-text'))$('#prompt-text').textContent=activeItem?.kind==='skill'?activeBody:renderPrompt(activeBody,Object.fromEntries([...document.querySelectorAll('[data-variable]')].map(el=>[el.dataset.variable,el.value]))); }
function scheduleSearch() { controller?.abort(); generation++; clearTimeout(searchTimer); busy=true; renderCards(); searchTimer=setTimeout(loadList,300); }
document.addEventListener('input',e=>{
  if (e.target.id==='search' && !e.isComposing) { query=e.target.value; offset=0; scheduleSearch(); }
  if (e.target.id==='model') { model=e.target.value;offset=0;scheduleSearch(); }
  if (e.target.matches('[data-variable]'))updatePreview();
});
document.addEventListener('compositionend',e=>{if(e.target.id==='search'){query=e.target.value;offset=0;scheduleSearch();}});
document.addEventListener('change',e=>{
  if(e.target.id==='language'){language=e.target.value;offset=0;loadList();}
  if(e.target.id==='mobile-category'){category=e.target.value;offset=0;renderCategories();loadList();}
  if(e.target.id==='repo') { if(!sources?.some(s=>s.value===e.target.value)){notify('请选择来源列表中的仓库');return;}repo=e.target.value;skillData=null;resetFilters();$('#search').value='';loadList(); }
});
document.addEventListener('click',async e=>{
  const el=e.target.closest('button,a');if(!el)return;
  if(el.dataset.launcherPreview){
    const variables=el.dataset.launcherPreview==='variables';
    $('#launcher-preview').src=variables?'assets/launcher.png':'assets/launcher-search.png';
    $('#launcher-preview').alt=variables?'CueTuck 独立启动器：填写变量并预览要复制的提示词':'CueTuck 独立启动器：搜索产品方案提示词，使用键盘选择结果';
    document.querySelectorAll('[data-launcher-preview]').forEach(b=>b.setAttribute('aria-pressed',String(b===el)));
  }
  if(el.matches('.skip')){e.preventDefault();$('#main').focus();return;}
  if(el.matches('[data-close]'))el.closest('dialog')?.close();
  if(el.matches('[data-create]'))$('#create').showModal();
  if(el.matches('[data-account]'))$('#account').showModal();
  if(el.hasAttribute('data-category')){category=el.dataset.category;offset=0;renderCategories();loadList();}
  if(el.dataset.open)openItem(el.dataset.open);
  if(el.hasAttribute('data-retry-detail'))openItem(activeItem.id);
  if(el.dataset.image){$('#image-viewer img').src=el.dataset.image;$('#image-viewer').showModal();}
  if(el.dataset.view){view=el.dataset.view;$('#cards').classList.toggle('list',view==='list');document.querySelectorAll('[data-view]').forEach(b=>b.classList.toggle('active',b.dataset.view===view));}
  if(el.dataset.sort){sort=el.dataset.sort;recommendation=null;offset=0;document.querySelectorAll('[data-sort]').forEach(b=>b.classList.toggle('active',b.dataset.sort===sort));loadList();}
  if(el.dataset.page){offset=el.dataset.page==='prev'?Math.max(0,offset-PAGE_SIZE):nextOffset;loadList();window.scrollTo(0,0);}
  if(el.hasAttribute('data-refresh')){if(route==='skills')repoCache.delete(repo);if(route==='app'){if(sort==='推荐')recentRecommendations=items.length?[...new Set([...recentRecommendations,...items.map(i=>i.id)])].slice(-48):[];recommendation=sort==='推荐'?crypto.randomUUID():null;offset=0;}loadList();}
  if(el.dataset.dismiss){if(dismissed.length>=200){notify('已保存 200 条，请先恢复推荐偏好。');return;}if(saveDismissed([...new Set([...dismissed,el.dataset.dismiss])])){offset=0;recommendation=null;loadList();notify('已设为不感兴趣，可恢复推荐偏好。');}}
  if(el.hasAttribute('data-restore-recommendations')&&saveDismissed([])){offset=0;recommendation=null;loadList();}
  if(el.hasAttribute('data-clear')){resetFilters();$('#main').innerHTML=workspace();fillDictionaries();loadList();}
  if(el.dataset.save){const p=activeItem?.id===el.dataset.save?activeItem:currentItem(el.dataset.save);if(!p)return;saved.has(p.id)?saved.delete(p.id):saved.set(p.id,p);notify(saved.has(p.id)?'已暂存到本页，刷新后清空。':'已取消暂存');if(route==='favorites')loadList();else renderCards();if($('[data-detail-save]'))$('[data-detail-save]').textContent=saved.has(p.id)?'取消暂存':'暂存到本页';}
  if(el.hasAttribute('data-copy')){el.disabled=true;try{await navigator.clipboard.writeText($('#prompt-text').textContent);notify('已复制提示词');}catch{notify('复制失败，请选择正文手动复制。');}finally{el.disabled=false;}}
});
document.addEventListener('error',e=>{if(e.target.matches?.('.card-cover img,.detail-image img')){e.target.closest('button').classList.add('image-failed');e.target.alt='图片暂时无法加载';}},true);
$('#create-form').addEventListener('submit',e=>{
  e.preventDefault();const data=new FormData(e.target), title=String(data.get('title')).trim(),content=String(data.get('content')).trim();if(!title||!content){notify('请填写标题和正文');return;}
  drafts.unshift({id:`draft-${crypto.randomUUID()}`,kind:'draft',title,content,model:'通用'});$('#create').close();e.target.reset();if(location.hash==='#drafts')render();else location.hash='drafts';notify('草稿仅保存到当前标签页。');
});
$('#detail').addEventListener('close',()=>{detailController?.abort();detailGeneration++;});
for(const dialog of document.querySelectorAll('dialog'))dialog.addEventListener('click',e=>{if(e.target===dialog){const r=dialog.getBoundingClientRect();if(e.clientX<r.left||e.clientX>r.right||e.clientY<r.top||e.clientY>r.bottom)dialog.close();}});
window.addEventListener('hashchange',render);render();
