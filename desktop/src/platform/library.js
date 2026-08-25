import { serializeCoverUrls } from "../lib/cover.js";

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

export function resetMemoryLibrary() {
  memoryPrompts = [];
  memoryCollections = [];
  memorySettings = { theme: "light" };
  memoryCategories = seedCategories();
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
  if (row.category_id === categoryId) return true;
  const category = categories.find((item) => item.id === row.category_id);
  return category?.parent_id === categoryId;
}

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function tauriInvoke(command, args) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command, args);
}

export async function createLocalPrompt({ title, content, categoryId = null, source = "local" } = {}) {
  if (isTauri()) {
    return tauriInvoke("create_local_prompt", {
      title,
      content,
      category_id: categoryId,
    });
  }
  const row = {
    id: `mem-${Date.now()}`,
    title: title.trim(),
    summary: null,
    content,
    category_id: categoryId,
    collection_id: null,
    use_count: 0,
    source,
    updated_at: String(Date.now()),
  };
  memoryPrompts.unshift(row);
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
    if (!(String(updatedAt ?? "") > String(existing.updated_at ?? "0"))) {
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

export async function importDownloadedPrompt({ title, content, remoteId = null, author = null } = {}) {
  const keptAuthor = await authorForDownload(author);
  if (isTauri()) {
    return tauriInvoke("import_downloaded_prompt", {
      title,
      content,
      remote_id: remoteId,
      author: keptAuthor,
    });
  }
  const row = {
    id: `dl-${Date.now()}`,
    title: String(title ?? "").trim(),
    summary: null,
    content: content ?? "",
    category_id: null,
    collection_id: null,
    use_count: 0,
    source: "downloaded",
    remote_id: remoteId,
    author: keptAuthor,
  };
  memoryPrompts.unshift(row);
  return row;
}

export async function updateLocalPrompt({ id, title, content, categoryId = null } = {}) {
  if (isTauri()) {
    return tauriInvoke("update_local_prompt", {
      id,
      title,
      content,
      category_id: categoryId,
    });
  }
  const row = memoryPrompts.find((item) => item.id === id);
  if (!row) throw new Error("提示词不存在");
  row.title = title.trim();
  row.content = content;
  row.category_id = categoryId;
  return row;
}

export async function deleteLocalPrompt(id) {
  if (isTauri()) {
    return tauriInvoke("delete_local_prompt", { id });
  }
  memoryPrompts = memoryPrompts.filter((item) => item.id !== id);
}

export async function listLocalPrompts({ query = "", categoryId = null } = {}) {
  if (isTauri()) {
    return tauriInvoke("list_local_prompts", {
      query,
      category_id: categoryId,
    });
  }
  return memoryPrompts.filter(
    (row) => matchesQuery(row, query, memoryCategories) && inCategory(row, categoryId, memoryCategories),
  );
}

export async function listLocalCategories() {
  if (isTauri()) {
    return tauriInvoke("list_local_categories");
  }
  return memoryCategories;
}

export async function createLocalCategory({ name, parentId } = {}) {
  const title = String(name ?? "").trim();
  if (!title) throw new Error("分类名称不能为空");
  if (isTauri()) {
    return tauriInvoke("create_local_category", { name: title, parentId });
  }
  const parent = memoryCategories.find((row) => row.id === parentId);
  if (!parent) throw new Error("大分类不存在");
  if (parent.parent_id) throw new Error("小分类下不能再创建子分类");
  const siblings = memoryCategories.filter((row) => row.parent_id === parentId);
  const row = {
    id: `cat-user-${memoryCategories.length + 1}`,
    parent_id: parentId,
    name: title,
    icon: null,
    is_system: false,
    sort_order: siblings.length,
  };
  memoryCategories.push(row);
  return row;
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
    id: `col-${Date.now()}`,
    title: title.trim(),
    description: null,
    category_id: categoryId,
    cover_type: coverType || "none",
    cover_json,
    member_count: 0,
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
    (row) => matchesQuery(row, query, memoryCategories) && inCategory(row, categoryId, memoryCategories),
  );
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
  if (!prompt || !collection) throw new Error("合集或提示词不存在");
  prompt.collection_id = collectionId;
  collection.member_count = memoryPrompts.filter((item) => item.collection_id === collectionId).length;
}

export async function listCollectionMembers(collectionId) {
  if (isTauri()) {
    return tauriInvoke("list_local_collection_members", { collection_id: collectionId });
  }
  return memoryPrompts.filter((item) => item.collection_id === collectionId);
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
}

export async function exportLocalLibrary() {
  if (isTauri()) {
    return tauriInvoke("export_local_library");
  }
  return JSON.stringify(
    {
      prompts: memoryPrompts.map(({ title, content }) => ({ title, content })),
      collections: memoryCollections.map(({ title }) => ({ title })),
    },
    null,
    2,
  );
}

export function previewImportJson(json) {
  const file = JSON.parse(json);
  const prompts = file.prompts ?? [];
  const collections = file.collections ?? [];
  return {
    prompt_count: prompts.length,
    collection_count: collections.length,
    titles: [...prompts.map((item) => item.title), ...collections.map((item) => item.title)],
  };
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
  const file = JSON.parse(json);
  for (const prompt of file.prompts ?? []) {
    await createLocalPrompt({ title: prompt.title, content: prompt.content ?? "" });
  }
  for (const collection of file.collections ?? []) {
    await createLocalCollection({ title: collection.title });
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
  return JSON.stringify({
    prompts: memoryPrompts.map(({ title, content }) => ({ title, content })),
    collections: memoryCollections.map(({ title }) => ({ title })),
    settings: memorySettings,
  });
}

export async function clearLocalPromptUse() {
  if (isTauri()) {
    return tauriInvoke("clear_local_prompt_use");
  }
  memoryPrompts.forEach((row) => {
    row.use_count = 0;
  });
}
