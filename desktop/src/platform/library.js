import { serializeCoverUrls } from "../lib/cover.js";
import { validateAssets, resetMemoryAssets, memoryAssets, storeMemoryAssets } from './assets.js';

const TONES = {
  软件开发: { icon: "</>", tone: "blue" },
  图片生成: { icon: "◇", tone: "violet" },
  视频创作: { icon: "▷", tone: "coral" },
  办公效率: { icon: "▤", tone: "amber" },
  内容写作: { icon: "✎", tone: "green" },
  产品设计: { icon: "◫", tone: "teal" },
  市场营销: { icon: "↗", tone: "rose" },
  数据分析: { icon: "▥", tone: "cyan" },
  教育学习: { icon: "♢", tone: "gold" },
  生活助手: { icon: "⌂", tone: "lime" },
};

const PRESET_CATEGORIES = [
  ["cat-software", "软件开发", ["网站开发", "前端工程", "后端与数据库", "测试与审查"]],
  ["cat-image", "图片生成", ["人像摄影", "商品视觉", "插画与海报"]],
  ["cat-video", "视频创作", ["分镜脚本", "短视频"]],
  ["cat-office", "办公效率", ["PPT 制作", "数据表格", "会议与邮件"]],
  ["cat-writing", "内容写作", ["社交媒体", "长文写作", "SEO"]],
  ["cat-product", "产品设计", ["PRD 与需求", "竞品分析", "用户研究"]],
  ["cat-marketing", "市场营销", ["品牌与广告", "增长运营", "销售话术"]],
  ["cat-data", "数据分析", ["SQL 与清洗", "业务洞察", "可视化"]],
  ["cat-education", "教育学习", ["课程与教案", "私人导师", "论文与研究"]],
  ["cat-life", "生活助手", ["旅行规划", "饮食与健身", "求职成长"]],
];

let memoryPrompts = [];
let memoryCollections = [];
let memorySettings = { theme: "light" };
let memoryCategories = seedCategories();
let memoryClock = 0;
let memorySettingStamps = {};
const SYNC_SETTINGS = ["theme", "default_model", "model_catalog", "custom_models", "show_model_tags", "ui_language", "variable_hints"];

export function timestampMillis(raw) {
  const value = Number(raw || 0);
  return value >= 1e9 && value < 1e11 ? value * 1000 : value;
}

function nextTimestamp() {
  memoryClock = Math.max(Date.now(), memoryClock + 1);
  return String(memoryClock);
}

function validatePayload(row) {
  for (const field of ["id", "key", "value_json", "title", "name", "icon", "summary", "description", "parent_id", "category_id", "collection_id", "content", "model", "source", "remote_id", "author", "cover_json", "cover_type", "created_at", "updated_at", "last_used_at", "deleted_at"]) {
    if (row[field] != null && typeof row[field] !== "string") throw new Error(`字段 ${field} 格式错误`);
  }
  for (const field of ["version", "use_count", "sort_order"]) {
    if (row[field] != null && (!Number.isSafeInteger(row[field]) || row[field] < 0)) throw new Error(`字段 ${field} 格式错误`);
  }
}

function nextMemoryId(prefix) {
  return `${prefix}-${crypto.randomUUID()}`;
}

export function resetMemoryLibrary() {
  resetMemoryAssets();
  memoryPrompts = [];
  memoryCollections = [];
  memorySettings = { theme: "light" };
  memoryCategories = seedCategories();
  memoryClock = 0;
  memorySettingStamps = {};
}

function seedCategories() {
  const rows = [];
  PRESET_CATEGORIES.forEach(([id, name, children], index) => {
    rows.push({
      id,
      parent_id: null,
      name,
      icon: null,
      is_system: true,
      sort_order: index,
    });
    children.forEach((child, childIndex) => {
      rows.push({
        id: `${id}-${childIndex}`,
        parent_id: id,
        name: child,
        icon: null,
        is_system: true,
        sort_order: childIndex,
      });
    });
  });
  return rows;
}

export function buildCategoryTree(records) {
  return records
    .filter((row) => !row.parent_id)
    .map((parent) => ({
      ...parent,
      open: parent.name === "软件开发" || parent.name === "图片生成",
      icon: TONES[parent.name]?.icon ?? "⌘",
      tone: TONES[parent.name]?.tone ?? "warm",
      children: records.filter((row) => row.parent_id === parent.id),
    }));
}

function matchesQuery(row, query, categories) {
  const needle = query.trim().toLowerCase();
  if (!needle) return true;
  const category = categories.find((item) => item.id === row.category_id);
  const parent = categories.find((item) => item.id === category?.parent_id);
  return `${row.title} ${row.content} ${category?.name ?? ""} ${parent?.name ?? ""}`
    .toLowerCase()
    .includes(needle);
}

function inCategory(row, categoryId, categories) {
  if (!categoryId) return true;
  if (categoryId === "__uncategorized__") return !categories.some((item) => item.id === row.category_id);
  if (row.category_id === categoryId) return true;
  const category = categories.find((item) => item.id === row.category_id);
  return category?.parent_id === categoryId;
}

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

import { invokeCommand as tauriInvoke } from "./tauri.js";

export async function createLocalPrompt({ title, content, categoryId = null, source = "local", model = null, assets } = {}) {
  if (assets !== undefined) validateAssets(assets);
  if (isTauri() && assets !== undefined) return tauriInvoke('save_local_prompt_with_assets', { id: null, title, content, category_id: categoryId, model, assets });
  if (isTauri()) {
    return tauriInvoke("create_local_prompt", {
      title,
      content,
      category_id: categoryId,
      model,
    });
  }
  const row = {
    id: nextMemoryId("mem"),
    title: title.trim(),
    summary: null,
    content,
    category_id: categoryId,
    collection_id: null,
    use_count: 0,
    source,
    model,
    last_used_at: null,
    updated_at: nextTimestamp(),
  };
  memoryPrompts.unshift(row);
  if (assets !== undefined) {
    storeMemoryAssets(row.id, assets);
    row.asset_count = assets.length; row.image_count = assets.filter(a => a.mime.startsWith('image/')).length;
  }
  return row;
}

export async function insertSyncedLocalPrompt({
  id,
  title,
  content,
  categoryId = null,
  updatedAt = "0",
} = {}) {
  const promptId = String(id ?? "").trim();
  const heading = String(title ?? "").trim();
  if (!promptId || !heading) throw new Error("同步提示词缺少 id 或标题");
  if (isTauri()) {
    return tauriInvoke("upsert_synced_local_prompt", {
      id: promptId,
      title: heading,
      content: content ?? "",
      category_id: categoryId,
      updated_at: String(updatedAt ?? "0"),
    });
  }
  const existing = memoryPrompts.find((item) => item.id === promptId);
  if (existing) {
    if (timestampMillis(updatedAt) <= timestampMillis(existing.updated_at)) {
      return existing;
    }
    existing.title = heading;
    existing.content = content ?? "";
    existing.category_id = categoryId;
    existing.updated_at = String(updatedAt ?? "0");
    return existing;
  }
  const row = {
    id: promptId,
    title: heading,
    summary: null,
    content: content ?? "",
    category_id: categoryId,
    collection_id: null,
    use_count: 0,
    source: "local",
    updated_at: String(updatedAt ?? "0"),
  };
  memoryPrompts.unshift(row);
  return row;
}

async function authorForDownload(author) {
  const keep = (await getLocalSetting("keep_author_on_download")) === "1";
  if (!keep) return null;
  const value = String(author ?? "").trim();
  return value || null;
}

export async function importDownloadedPrompt({ title, content, remoteId = null, author = null, categoryId = null, model = null } = {}) {
  const keptAuthor = await authorForDownload(author);
  if (isTauri()) {
    return tauriInvoke("import_downloaded_prompt", {
      title,
      content,
      remote_id: remoteId,
      author: keptAuthor,
      category_id: categoryId,
      model,
    });
  }
  const row = {
    id: nextMemoryId("dl"),
    title: String(title ?? "").trim(),
    summary: null,
    content: content ?? "",
    category_id: categoryId,
    collection_id: null,
    use_count: 0,
    source: "downloaded",
    remote_id: remoteId,
    author: keptAuthor,
    model,
    updated_at: nextTimestamp(),
  };
  memoryPrompts.unshift(row);
  return row;
}

export async function appendDownloadedAssets(id, remoteId, additions) {
  validateAssets(additions);
  if (isTauri()) return tauriInvoke('append_downloaded_assets', {prompt_id:id,remote_id:remoteId,assets:additions});
  const row = memoryPrompts.find(row => row.id === id && !row.deleted_at && row.remote_id === remoteId);
  if (!row) throw Error('本地副本不存在或来源已变化');
  const assets = memoryAssets(id), before = assets.length;
  for (const asset of additions) if (!assets.some(old => old.mime === asset.mime && old.data === asset.data)) assets.push({...asset,id:crypto.randomUUID()});
  validateAssets(assets);
  if (assets.length > before) {
    storeMemoryAssets(id,assets); row.asset_count=assets.length; row.image_count=assets.filter(a=>a.mime.startsWith('image/')).length; row.updated_at=String(Date.now());
  }
  return structuredClone(row);
}

export async function updateLocalPrompt({ id, title, content, categoryId = null, model, assets } = {}) {
  if (assets !== undefined) validateAssets(assets);
  if (isTauri() && assets !== undefined) return tauriInvoke('save_local_prompt_with_assets', { id, title, content, category_id: categoryId, model, assets });
  if (isTauri()) {
    return tauriInvoke("update_local_prompt", {
      id,
      title,
      content,
      category_id: categoryId,
      model,
    });
  }
  const row = memoryPrompts.find((item) => item.id === id);
  if (!row) throw new Error("提示词不存在");
  row.title = title.trim();
  row.content = content;
  row.category_id = categoryId;
  if (model !== undefined) row.model = model;
  row.updated_at = nextTimestamp();
  if (assets !== undefined) {
    storeMemoryAssets(id, assets);
    row.asset_count = assets.length; row.image_count = assets.filter(a => a.mime.startsWith('image/')).length;
  }
  return row;
}

export async function deleteLocalPrompt(id) {
  if (isTauri()) {
    return tauriInvoke("delete_local_prompt", { id });
  }
  const row = memoryPrompts.find((item) => item.id === id);
  if (row) row.deleted_at = row.updated_at = nextTimestamp();
}

export async function listLocalPrompts({ query = "", categoryId = null } = {}) {
  if (isTauri()) {
    return tauriInvoke("list_local_prompts", {
      query,
      category_id: categoryId,
    });
  }
  return memoryPrompts.filter(
    (row) => !row.deleted_at && matchesQuery(row, query, memoryCategories) && inCategory(row, categoryId, memoryCategories),
  );
}

export async function listLocalCategories() {
  if (isTauri()) {
    return tauriInvoke("list_local_categories");
  }
  return memoryCategories.filter(row => !row.deleted_at);
}

export async function createLocalCategory({ name, parentId = null } = {}) {
  const title = String(name ?? "").trim();
  if (!title) throw new Error("分类名称不能为空");
  if (isTauri()) {
    return tauriInvoke("create_local_category", { name: title, parentId });
  }
  if (parentId !== null) {
    const parent = memoryCategories.find((row) => row.id === parentId && !row.deleted_at);
    if (!parent) throw new Error("大分类不存在");
    if (parent.parent_id) throw new Error("小分类下不能再创建子分类");
  }
  const siblings = memoryCategories.filter((row) => row.parent_id === parentId && !row.deleted_at);
  if (siblings.some(row => row.name.trim() === title)) throw new Error("同一级已有同名分类");
  const row = {
    id: nextMemoryId("cat-user"),
    parent_id: parentId,
    name: title,
    icon: null,
    is_system: false,
    sort_order: siblings.length,
    updated_at: nextTimestamp(),
  };
  memoryCategories.push(row);
  return row;
}

export async function deleteLocalCategory(id) {
  if (isTauri()) return tauriInvoke('delete_local_category', { id });
  const category = memoryCategories.find(row => row.id === id && !row.deleted_at);
  if (!category) throw new Error('分类不存在');
  if (category.is_system) throw new Error('系统分类不能删除');
  if (memoryCategories.some(row => row.parent_id === id && !row.deleted_at)) throw new Error('请先删除该大分类下的小分类');
  const timestamp = nextTimestamp();
  category.deleted_at = category.updated_at = timestamp;
  for (const row of [...memoryPrompts, ...memoryCollections]) {
    if (row.category_id === id) { row.category_id = null; row.updated_at = timestamp; }
  }
}

export async function createLocalCollection({
  title,
  categoryId = null,
  coverType = "none",
  coverUrls = [],
} = {}) {
  const cover_json = serializeCoverUrls(coverType, coverUrls);
  if (isTauri()) {
    return tauriInvoke("create_local_collection", {
      title,
      category_id: categoryId,
      cover_type: coverType,
      cover_json,
    });
  }
  const row = {
    id: nextMemoryId("col"),
    title: title.trim(),
    description: null,
    category_id: categoryId,
    cover_type: coverType || "none",
    cover_json,
    member_count: 0,
    updated_at: nextTimestamp(),
  };
  memoryCollections.unshift(row);
  return row;
}

export async function listLocalCollections({ query = "", categoryId = null } = {}) {
  if (isTauri()) {
    return tauriInvoke("list_local_collections", {
      query,
      category_id: categoryId,
    });
  }
  return memoryCollections.filter(
    (row) => !row.deleted_at && matchesQuery(row, query, memoryCategories) && inCategory(row, categoryId, memoryCategories),
  ).map((row) => ({ ...row, member_count: memoryPrompts.filter((prompt) => !prompt.deleted_at && prompt.collection_id === row.id).length }));
}

export async function addPromptToCollection(promptId, collectionId) {
  if (isTauri()) {
    return tauriInvoke("add_prompt_to_local_collection", {
      prompt_id: promptId,
      collection_id: collectionId,
    });
  }
  const prompt = memoryPrompts.find((item) => item.id === promptId);
  const collection = memoryCollections.find((item) => item.id === collectionId);
  if (!prompt || prompt.deleted_at || !collection || collection.deleted_at) throw new Error("合集或提示词不存在");
  prompt.collection_id = collectionId;
  prompt.updated_at = nextTimestamp();
  collection.member_count = memoryPrompts.filter((item) => item.collection_id === collectionId).length;
}

export async function removePromptFromCollection(promptId, collectionId) {
  if (isTauri()) return tauriInvoke("remove_prompt_from_local_collection", { prompt_id: promptId, collection_id: collectionId });
  const row = memoryPrompts.find((item) => item.id === promptId && item.collection_id === collectionId && !item.deleted_at);
  if (row) { row.collection_id = null; row.updated_at = nextTimestamp(); }
}

export async function updateLocalCollection({ id, title, categoryId = null, coverType = "none", coverUrls = [] }) {
  const cover_json = serializeCoverUrls(coverType, coverUrls);
  if (isTauri()) return tauriInvoke("update_local_collection", { id, title, category_id: categoryId, cover_type: coverType, cover_json });
  const row = memoryCollections.find((item) => item.id === id && !item.deleted_at);
  if (!row) throw new Error("合集不存在");
  if (!title.trim()) throw new Error("合集名称不能为空");
  Object.assign(row, { title: title.trim(), category_id: categoryId, cover_type: coverType, cover_json, updated_at: nextTimestamp() });
}

export async function deleteLocalCollection(id) {
  if (isTauri()) return tauriInvoke("delete_local_collection", { id });
  const row = memoryCollections.find((item) => item.id === id && !item.deleted_at);
  if (!row) return;
  row.deleted_at = row.updated_at = nextTimestamp();
  for (const prompt of memoryPrompts.filter((item) => item.collection_id === id)) {
    prompt.collection_id = null;
    prompt.updated_at = row.updated_at;
  }
}

export async function listCollectionMembers(collectionId) {
  if (isTauri()) {
    return tauriInvoke("list_local_collection_members", { collection_id: collectionId });
  }
  return memoryPrompts.filter((item) => !item.deleted_at && item.collection_id === collectionId);
}

export async function getLocalSetting(key) {
  if (isTauri()) {
    try {
      return await tauriInvoke("get_local_setting", { key });
    } catch {
      return key === "theme" ? "light" : "";
    }
  }
  return memorySettings[key] ?? "";
}

export async function setLocalSetting(key, value) {
  if (isTauri()) {
    return tauriInvoke("set_local_setting", { key, value });
  }
  memorySettings[key] = value;
  memorySettingStamps[key] = nextTimestamp();
}

export async function exportLocalSyncChanges({ includeAssets = false } = {}) {
  if (isTauri()) return tauriInvoke("export_local_sync_changes", { include_assets: includeAssets });
  const rows = [
    ...memoryCategories.map((row) => ["category", row]),
    ...memoryCollections.map((row) => ["collection", row]),
    ...memoryPrompts.map((row) => ["prompt", row]),
    ...SYNC_SETTINGS.filter((key) => key in memorySettings).map((key) => ["setting", {
      id: `setting:${key}`, key, value_json: JSON.stringify(memorySettings[key]), updated_at: memorySettingStamps[key] ?? "0",
    }]),
  ];
  return rows.map(([kind, row]) => ({
    id: row.id, kind, payload: { ...row, ...(includeAssets && kind === 'prompt' && !row.deleted_at ? { assets: memoryAssets(row.id) } : {}) }, updated_at: String(timestampMillis(row.updated_at)), deleted_at: row.deleted_at ?? null,
  }));
}

export async function applyLocalSyncChanges(items, { keepLocal = false, includeAssets = false } = {}) {
  if (isTauri()) return tauriInvoke("apply_local_sync_changes", { items, keep_local: keepLocal, include_assets: includeAssets });
  if (items.some((item) => !["category", "collection", "prompt", "setting"].includes(item.kind) || !/^\d+$/.test(item.updated_at))) throw new Error("同步记录类型或时间无效");
  const tables = { category: memoryCategories.map((row) => ({ ...row })), collection: memoryCollections.map((row) => ({ ...row })), prompt: memoryPrompts.map((row) => ({ ...row })) };
  const settings = { ...memorySettings };
  const stamps = { ...memorySettingStamps };
  const pendingAssets = new Map();
  for (const kind of ["category", "collection", "prompt", "setting"]) {
    const ordered = items.filter((row) => row.kind === kind);
    if (kind === 'category') ordered.sort((a, b) => Number(a.payload?.parent_id != null) - Number(b.payload?.parent_id != null));
    for (const item of ordered) {
      if (!item.id || !item.payload || typeof item.payload !== "object" || Array.isArray(item.payload)) throw new Error("同步记录格式错误");
      const payload = { ...item.payload };
      delete payload.assets; delete payload.asset_refs; delete payload.asset_count; delete payload.image_count;
      validatePayload(payload);
      if (kind === "setting") {
        const key = item.id.replace(/^setting:/, "");
        if (SYNC_SETTINGS.includes(key) && timestampMillis(item.updated_at) > timestampMillis(stamps[key])) {
          const value = JSON.parse(payload.value_json);
          if (typeof value !== "string") throw new Error("设置格式错误");
          settings[key] = value;
          stamps[key] = item.updated_at;
        }
        continue;
      }
      const existing = tables[kind].find((row) => row.id === item.id);
      if (includeAssets && kind === 'prompt' && !item.deleted_at && item.payload.assets !== undefined
        && (!existing || (!keepLocal && timestampMillis(existing.updated_at) <= timestampMillis(item.updated_at)))) {
        validateAssets(item.payload.assets);
        const merged = pendingAssets.get(item.id) ?? memoryAssets(item.id);
        for (const asset of item.payload.assets) if (!merged.some(a => a.id === asset.id)) merged.push(asset);
        validateAssets(merged);
        pendingAssets.set(item.id, merged);
        if (existing) { existing.asset_count = merged.length; existing.image_count = merged.filter(a => a.mime.startsWith('image/')).length; }
      }
      if (existing && kind === "collection" && timestampMillis(existing.updated_at) === timestampMillis(item.updated_at)) {
        for (const field of ["cover_json", "cover_type"]) if (field in payload) existing[field] = payload[field];
      }
      if (existing && ((keepLocal && kind === "prompt") || timestampMillis(existing.updated_at) >= timestampMillis(item.updated_at))) continue;
      if (kind === "category") {
        if (existing?.is_system) continue;
        if (!payload.name?.trim() || (payload.parent_id != null && !tables.category.some((row) => row.id === payload.parent_id && !row.parent_id))) throw new Error("同步分类无效");
      }
      for (const [field, target] of [["category_id", "category"], ["collection_id", "collection"]]) {
        if (payload[field] != null && !tables[target].some((row) => row.id === payload[field])) throw new Error(`同步记录引用不存在的 ${field}`);
      }
      const row = { ...existing, ...payload, id: item.id, updated_at: String(timestampMillis(item.updated_at)) };
      if (kind === "category") { row.is_system = false; row.parent_id = payload.parent_id ?? null; }
      row.deleted_at = item.deleted_at ?? null;
      if (existing) Object.assign(existing, row);
      else tables[kind].push(row);
    }
  }
  if (tables.category.some(row => row.parent_id != null && !tables.category.some(parent => parent.id === row.parent_id && parent.parent_id == null && (row.deleted_at || !parent.deleted_at)))) throw new Error('同步分类层级无效');
  const deletedCategories = new Set(tables.category.filter(row => row.deleted_at).map(row => row.id));
  for (const row of [...tables.prompt, ...tables.collection]) {
    if (deletedCategories.has(row.category_id)) row.category_id = null;
  }
  memoryCategories = tables.category;
  memoryCollections = tables.collection;
  memoryPrompts = tables.prompt;
  memorySettings = settings;
  memorySettingStamps = stamps;
  for (const [id, assets] of pendingAssets) {
    storeMemoryAssets(id, assets);
    const row = memoryPrompts.find(row => row.id === id);
    row.asset_count = assets.length; row.image_count = assets.filter(a => a.mime.startsWith('image/')).length;
  }
  for (const item of items) memoryClock = Math.max(memoryClock, timestampMillis(item.updated_at));
}

export async function exportLocalLibrary() {
  if (isTauri()) {
    return tauriInvoke("export_local_library");
  }
  return JSON.stringify(
    {
      version: 2,
      prompts: memoryPrompts.filter((row) => !row.deleted_at).map(row => ({ ...row, assets: memoryAssets(row.id) })),
      collections: memoryCollections.filter((row) => !row.deleted_at),
      categories: memoryCategories.filter(row => !row.deleted_at),
    },
    null,
    2,
  );
}

export function previewImportJson(json) {
  const { file } = prepareImport(json, "0");
  const prompts = file.prompts;
  const collections = file.collections;
  return {
    prompt_count: prompts.length,
    collection_count: collections.length,
    titles: [...prompts.map((item) => item.title), ...collections.map((item) => item.title)],
  };
}

function prepareImport(json, timestamp) {
  const file = JSON.parse(json);
  if (!file || typeof file !== "object" || Array.isArray(file) || (file.version != null && ![1, 2].includes(file.version))) throw new Error("不支持的导入格式");
  const systemIds = new Set(seedCategories().map((row) => row.id));
  const rootIds = new Set(PRESET_CATEGORIES.map(([id]) => id));
  const maps = { category: new Map(), collection: new Map(), prompt: new Map() };
  const groups = [["category", "categories"], ["collection", "collections"], ["prompt", "prompts"]];
  for (const [kind, key] of groups) {
    if (!(key in file)) file[key] = [];
    if (!Array.isArray(file[key])) throw new Error(`${key} 必须是数组`);
    for (const row of file[key]) {
      const title = kind === "category" ? row?.name : row?.title;
      if (!row || typeof row !== "object" || Array.isArray(row) || typeof title !== "string" || !title.trim()) throw new Error("导入记录缺少名称");
      validatePayload(row);
      if (kind === 'prompt') validateAssets(row.assets ?? []);
      if (kind === "category" && !row.id) throw new Error("分类缺少 id");
      if (row.id) {
        if (maps[kind].has(row.id)) throw new Error("导入记录 id 重复");
        maps[kind].set(row.id, kind === "category" && systemIds.has(row.id) ? row.id : crypto.randomUUID());
      }
    }
  }
  const reference = (kind, id) => {
    if (id == null) return null;
    const mapped = maps[kind].get(id) ?? (kind === "category" && systemIds.has(id) ? id : null);
    if (!mapped) throw new Error(`导入记录引用不存在的 ${kind}`);
    return mapped;
  };
  for (const row of file.categories) {
    if (!systemIds.has(row.id) && row.parent_id == null) rootIds.add(row.id);
  }
  const changes = [];
  for (const [kind, key] of groups) {
    for (const row of file[key]) {
      if (kind === "category" && systemIds.has(row.id)) continue;
      const id = maps[kind].get(row.id) ?? crypto.randomUUID();
      const payload = { ...row, id, updated_at: timestamp, deleted_at: null };
      if (kind === "category") {
        if (row.parent_id != null && !rootIds.has(row.parent_id)) throw new Error("导入小分类必须属于大分类，最多两级");
        payload.parent_id = reference("category", row.parent_id);
        payload.is_system = false;
      } else {
        payload.title = row.title.trim();
        payload.category_id = reference("category", row.category_id);
        payload.created_at = timestamp;
        if (kind === "prompt") {
          payload.collection_id = reference("collection", row.collection_id);
          payload.content = row.content ?? "";
          payload.source = row.source ?? "local";
        } else {
          payload.cover_type = row.cover_type ?? "none";
          payload.cover_json = row.cover_json ?? "[]";
          const covers = JSON.parse(payload.cover_json);
          if (!Array.isArray(covers) || covers.some((url) => typeof url !== "string")) throw new Error("导入封面格式错误");
        }
      }
      changes.push({ id, kind, payload, updated_at: timestamp, deleted_at: null });
    }
  }
  return { file, changes };
}

export async function previewLocalImport(json) {
  if (isTauri()) {
    return tauriInvoke("preview_local_import", { json });
  }
  return previewImportJson(json);
}

export async function applyLocalImport(json) {
  if (isTauri()) {
    return tauriInvoke("apply_local_import", { json });
  }
  const preview = previewImportJson(json);
  const { changes } = prepareImport(json, nextTimestamp());
  await applyLocalSyncChanges(changes);
  for (const change of changes.filter(c => c.kind === 'prompt')) {
    const assets = change.payload.assets ?? [];
    storeMemoryAssets(change.id, assets);
    const row = memoryPrompts.find(p => p.id === change.id);
    row.asset_count = assets.length; row.image_count = assets.filter(a => a.mime.startsWith('image/')).length;
  }
  return preview;
}

export const FILE_BACKUP_DESKTOP_ONLY = "仅桌面窗口支持库文件备份";

export async function backupLocalLibrary(dest) {
  if (isTauri()) {
    return tauriInvoke("backup_local_library", { dest: dest || null });
  }
  throw new Error(FILE_BACKUP_DESKTOP_ONLY);
}

export async function setAutoBackup(enabled) {
  if (isTauri()) return tauriInvoke("set_auto_backup", { enabled });
  throw new Error(FILE_BACKUP_DESKTOP_ONLY);
}

export async function restoreLocalLibrary(src) {
  if (isTauri()) {
    return tauriInvoke("restore_local_library", { src });
  }
  throw new Error(FILE_BACKUP_DESKTOP_ONLY);
}

export async function recordLocalPromptUse(id) {
  if (isTauri()) {
    return tauriInvoke("record_local_prompt_use", { id });
  }
  const row = memoryPrompts.find((item) => item.id === id);
  if (!row) throw new Error("提示词不存在");
  row.use_count = (row.use_count ?? 0) + 1;
  row.last_used_at = String(Date.now());
  row.updated_at = nextTimestamp();
  return row;
}

export async function openLibraryDir() {
  if (isTauri()) {
    return tauriInvoke("open_library_dir");
  }
  return "memory-library";
}

export async function exportLibraryZip() {
  if (isTauri()) {
    return tauriInvoke("export_library_zip", { dest: null });
  }
  return JSON.stringify({ ...JSON.parse(await exportLocalLibrary()), settings: memorySettings });
}

export async function clearLocalPromptUse() {
  if (isTauri()) {
    return tauriInvoke("clear_local_prompt_use");
  }
  memoryPrompts.forEach((row) => {
    row.use_count = 0;
    row.last_used_at = null;
    row.updated_at = nextTimestamp();
  });
}


export async function moveLocalPromptCategory(id, categoryId) {
  if (isTauri()) return tauriInvoke('move_local_prompt_category', { id, category_id: categoryId });
  const row = memoryPrompts.find(item => item.id === id && !item.deleted_at);
  if (!row) throw new Error('提示词不存在');
  if (categoryId && !memoryCategories.some(item => item.id === categoryId && !item.deleted_at)) throw new Error('分类不存在');
  row.category_id = categoryId; row.updated_at = nextTimestamp();
}
