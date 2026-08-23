<template>
  <div class="app-shell">
    <header
      data-region="titlebar"
      class="titlebar"
      :class="{ 'host-mac': host === 'macos' }"
      :style="{ '--traffic-light-inset': `${trafficInset}px` }"
      data-tauri-drag-region
    >
      <div class="titlebar-left">
        <button type="button" class="app-mark" aria-label="提示方舟">P</button>
      </div>
      <div class="titlebar-center" data-tauri-drag-region>
        <span class="brand-name">提示方舟</span>
        <span class="title-dot">·</span>
        <span>{{ locationLabel }}</span>
      </div>
      <div class="titlebar-right">
        <button type="button" class="title-tool" title="快捷搜索" @click="$emit('open-launcher')">
          <span>⌕</span><span>搜索</span><kbd>{{ shortcutLabel }}</kbd>
        </button>
        <button
          type="button"
          class="preference-toggle"
          :title="dark ? '切换浅色主题' : '切换深色主题'"
          @click="toggleTheme"
        >
          ◐
        </button>
        <button type="button" class="preference-toggle language-toggle" title="第一期仅中文">EN</button>
        <button
          type="button"
          class="account-button"
          data-testid="open-login"
          :title="session.loggedIn ? session.email : '登录'"
          @click="openLogin('登录账号')"
        >
          <span class="avatar">{{ session.loggedIn ? (session.email?.[0] || "已") : "游" }}</span>
          <span>{{ session.loggedIn ? "已登录" : "登录" }}</span>
        </button>
      </div>
    </header>

    <div class="workspace">
      <aside data-region="sidebar" class="sidebar">
        <div class="space-switch" role="tablist" aria-label="提示词空间">
          <button
            type="button"
            class="space-tab"
            data-space="square"
            role="tab"
            :class="{ active: space === 'square' }"
            @click="openSquare"
          >
            <span class="nav-icon">◎</span><span>提示词广场</span>
          </button>
          <button
            type="button"
            class="space-tab"
            data-space="local"
            role="tab"
            :class="{ active: space === 'local' }"
            @click="openLocal"
          >
            <span class="nav-icon">▣</span><span>本地提示词</span>
            <span class="nav-count">{{ localCount }}</span>
          </button>
        </div>

        <div class="sidebar-toolbar">
          <span>{{ space === "local" ? "我的分类" : "探索分类" }}</span>
          <div>
            <button type="button" class="mini-button" title="全部折叠" @click="collapseAll">−</button>
            <button type="button" class="mini-button" data-testid="add-category" title="新建小分类" @click="startAddCategory">＋</button>
          </div>
        </div>

        <div v-if="addingCategoryId" class="add-category">
          <input
            v-model="newCategoryName"
            data-testid="new-category-name"
            placeholder="小分类名称"
            @keydown.enter.prevent="confirmAddCategory"
          >
          <button type="button" data-testid="confirm-category" @click="confirmAddCategory">添加</button>
        </div>
        <p v-if="categoryError" data-testid="category-error">{{ categoryError }}</p>
        <nav class="category-tree" aria-label="提示词分类">
          <button
            type="button"
            class="tree-row"
            :class="{ active: !selectedId }"
            @click="selectCategory(null)"
          >
            <span class="chevron ghost">›</span>
            <span class="tree-icon warm">⌘</span>
            <span>全部提示词</span>
            <span class="tree-count">{{ localCount }}</span>
          </button>
          <div v-for="group in categoryGroups" :key="group.id" class="tree-group" :class="{ open: group.open }">
            <button type="button" class="tree-row tree-parent" :class="{ active: selectedId === group.id }" @click="toggleGroup(group)">
              <span class="chevron">›</span>
              <span class="tree-icon" :class="group.tone">{{ group.icon }}</span>
              <span>{{ group.name }}</span>
              <span class="tree-count">{{ group.children.length }}</span>
            </button>
            <div class="tree-children">
              <button
                v-for="child in group.children"
                :key="child.id"
                type="button"
                class="tree-row child"
                :class="{ active: selectedId === child.id }"
                @click="selectCategory(child.id)"
              >
                <span>{{ child.name }}</span>
              </button>
            </div>
          </div>
        </nav>

        <div class="sidebar-bottom">
          <button type="button">
            <span class="connection-dot offline"></span>
            <span>本地模式</span>
            <span class="sidebar-bottom-action">第一期</span>
          </button>
          <button type="button" data-testid="open-settings" @click="settingsOpen = true">
            <span>⚙</span><span>设置</span><span class="sidebar-bottom-action">›</span>
          </button>
        </div>
      </aside>

      <main data-region="content" class="content-area">
        <section class="content-header">
          <div class="content-heading">
            <p class="eyebrow">{{ space === "square" ? "COMMUNITY LIBRARY" : "LOCAL LIBRARY" }}</p>
            <h1>{{ space === "square" ? "发现好用的提示词" : "我的提示词" }}</h1>
            <p>
              {{
                space === "square"
                  ? "从社区创作者的实践中寻找灵感，下载后可离线编辑与使用。"
                  : "所有内容保存在本机，即使断网也可以继续编辑、整理与使用。"
              }}
            </p>
          </div>
          <div class="content-actions">
            <button v-if="space === 'square'" type="button" class="button ghost-button" @click="loadSquare">↻ 刷新</button>
            <button
              v-if="space === 'square'"
              type="button"
              class="button primary-button"
              data-testid="publish-prompt"
              @click="startPublish"
            >
              <span>＋</span><span>发布提示词</span>
            </button>
            <button
              v-else
              type="button"
              class="button primary-button"
              @click="creating = true"
            >
              <span>＋</span><span>新建</span>
            </button>
          </div>
        </section>

        <section class="filter-bar">
          <label class="inline-search">
            <span>⌕</span>
            <input
              v-model="query"
              type="search"
              :placeholder="space === 'square' ? '搜索标题、标签或作者' : '搜索标题或正文'"
              @input="space === 'square' ? loadSquare() : reloadPrompts()"
            >
            <kbd>/</kbd>
          </label>
          <div class="filter-tabs" role="tablist">
            <button
              v-for="tab in filterTabs"
              :key="tab"
              type="button"
              :class="{ active: sortTab === tab }"
              @click="setSort(tab)"
            >
              {{ tab }} <small>0</small>
            </button>
          </div>
          <div class="filter-spacer"></div>
          <label class="compact-select">
            <span>模型</span>
            <select><option>全部模型</option></select>
          </label>
          <div class="view-switch" aria-label="视图切换">
            <button type="button" :class="{ active: view === 'grid' }" title="网格视图" @click="view = 'grid'">▦</button>
            <button type="button" :class="{ active: view === 'list' }" title="列表视图" @click="view = 'list'">☷</button>
          </div>
        </section>

        <div
          v-if="space === 'square' && squareOffline"
          data-testid="square-offline"
          class="offline-banner"
        >
          <span>◌</span>
          <div>
            <strong>当前离线</strong>
            <small>广场列表暂时不可用，本地库仍可使用。</small>
          </div>
          <button type="button" data-testid="go-local" @click="openLocal">前往本地</button>
        </div>
        <div
          v-if="space === 'square' && squareBlocked"
          data-testid="square-blocked"
          class="offline-banner"
        >
          <span>◌</span>
          <div>
            <strong>已关闭广场访问</strong>
            <small>未请求广场接口。启动器仍只搜本地。</small>
          </div>
          <button type="button" data-testid="go-local" @click="openLocal">前往本地</button>
        </div>

        <section class="prompt-section">
          <div class="section-heading-row">
            <div>
              <span class="section-kicker">{{ space === "square" ? "TRENDING PROMPTS" : "LIBRARY" }}</span>
              <h2>{{ space === "square" ? "正在流行" : selectedLabel }}</h2>
            </div>
            <span class="result-count">共 {{ displayedItems.length }} 个结果</span>
          </div>
          <div
            v-if="displayedItems.length"
            class="prompt-grid"
            :class="{ 'list-view': view === 'list' }"
            data-testid="library-view"
            :data-layout="view"
          >
            <article
              v-for="item in displayedItems"
              :key="item.kind + item.id"
              class="prompt-card"
              :class="{ collection: item.kind === 'collection', 'as-row': view === 'list' }"
              @click="openItem(item)"
            >
              <div
                v-if="item.kind === 'collection' && coverPreview(item).length"
                class="collection-card-preview"
                data-testid="collection-cover-preview"
              >
                <img v-for="(src, index) in coverPreview(item)" :key="index" :src="src" alt="">
              </div>
              <div class="card-top">
                <span class="type-badge">{{ item.kind === "collection" ? "合集" : space === "square" ? "广场" : "本地" }}</span>
              </div>
              <h3>{{ item.title }}</h3>
              <p class="prompt-excerpt">
                {{
                  item.kind === "collection"
                    ? `${item.member_count ?? 0} 个提示词`
                    : item.content || item.excerpt || "还没有正文"
                }}
              </p>
              <div v-if="item.kind === 'prompt'" class="card-footer">
                <template v-if="space === 'square'">
                  <button
                    type="button"
                    class="card-action"
                    data-testid="download-square"
                    @click.stop="downloadSquare(item)"
                  >
                    下载
                  </button>
                  <button
                    type="button"
                    class="card-action"
                    data-testid="favorite-square"
                    @click.stop="favoriteSquare(item)"
                  >
                    {{ favoriteIds.includes(item.id) ? "已收藏" : "收藏" }}
                  </button>
                </template>
                <button v-else type="button" class="card-action" @click.stop="using = item">使用</button>
              </div>
            </article>
          </div>
          <div v-else class="empty-state">
            <span class="empty-glyph">{{ space === "square" ? "◎" : "▣" }}</span>
            <h3>{{ space === "square" ? (squareOffline ? "暂时看不到广场列表" : "广场还没有内容") : "本地库是空的" }}</h3>
            <p>{{ space === "square" ? "本地提示词仍然可用。" : "点右上角「新建提示词」即可写入本机。" }}</p>
          </div>
        </section>
      </main>
    </div>

    <CreatePromptModal
      v-if="creating || editing"
      :prompt="editing"
      :groups="categoryGroups"
      @cancel="closeEditor"
      @save="savePrompt"
      @remove="removePrompt"
    />
    <UsePromptModal v-if="using" :prompt="using" @cancel="using = null" @copied="finishUse" />
    <CollectionDetailModal
      v-if="openedCollection"
      :collection="openedCollection"
      :members="collectionMembers"
      :prompts="prompts"
      @cancel="openedCollection = null"
      @add="addToOpenedCollection"
    />
    <SettingsModal
      v-if="settingsOpen"
      :theme="theme"
      :host="host"
      :session="session"
      @cancel="settingsOpen = false"
      @theme="applyTheme"
      @imported="reloadPrompts"
      @login="openLogin('登录账号')"
      @logout="logoutFromSettings"
    />
    <LoginModal
      v-if="loginReason"
      :reason="loginReason"
      @cancel="loginReason = ''"
      @success="finishLogin"
    />
    <div v-if="publishResume" class="modal-layer" data-testid="publish-resume">
      <div class="modal-backdrop" @click="publishResume = false"></div>
      <section class="modal create-modal" role="dialog" aria-modal="true">
        <header class="modal-header">
          <div>
            <p class="modal-kicker">PUBLISH</p>
            <h2>发布到广场</h2>
          </div>
          <button type="button" class="modal-close" aria-label="关闭" @click="publishResume = false">×</button>
        </header>
        <div class="create-body">
          <label class="field">
            <span>本地内容</span>
            <select v-model="publishSourceId" data-testid="publish-source">
              <option value="">选择要发布的本地提示词或合集</option>
              <option v-for="item in publishSources" :key="item.id" :value="item.id">
                {{ item.title }}
              </option>
            </select>
          </label>
          <p>提交后本地正文仍可编辑，审核状态不会覆盖本机内容。</p>
        </div>
        <footer class="modal-footer">
          <button type="button" class="button ghost-button" @click="publishResume = false">关闭</button>
          <button
            type="button"
            class="button primary-button"
            data-testid="publish-submit"
            :disabled="!publishSourceId"
            @click="submitPublish"
          >
            提交审核
          </button>
        </footer>
      </section>
    </div>

    <footer data-region="statusbar" class="statusbar">
      <span class="status-item">
        <span class="connection-dot" :class="databaseStatus === 'ready' ? 'online' : 'offline'"></span>
        <span>本地优先</span>
      </span>
      <span class="status-sep"></span>
      <span class="status-item">{{ databaseLabel }}</span>
      <span class="status-item">本地 <strong>{{ localCount }}</strong> 条</span>
      <span class="status-spacer"></span>
      <span class="status-item muted-status">右键查看更多操作</span>
      <span class="status-sep"></span>
      <button type="button" class="status-button" @click="$emit('open-launcher')">
        ⌕ 快捷搜索 <kbd>{{ shortcutLabel }}</kbd>
      </button>
    </footer>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from "vue";
import CollectionDetailModal from "./CollectionDetailModal.vue";
import CreatePromptModal from "./CreatePromptModal.vue";
import LoginModal from "./LoginModal.vue";
import SettingsModal from "./SettingsModal.vue";
import UsePromptModal from "./UsePromptModal.vue";
import { getSession, logoutSession } from "../platform/session.js";
import { createPublication, deleteFavorite, downloadSquareItem, listFavorites, listSquareItems, putFavorite } from "../platform/square.js";
import { parseCoverUrls } from "../lib/cover.js";
import { DEFAULT_LAUNCHER_SHORTCUT } from "../platform/shortcut.js";
import { applyHostChrome, detectHost, formatShortcutLabel, trafficLightInsetPx } from "../platform/windowChrome.js";
import {
  addPromptToCollection,
  buildCategoryTree,
  createLocalCategory,
  createLocalCollection,
  createLocalPrompt,
  deleteLocalPrompt,
  getLocalSetting,
  listCollectionMembers,
  listLocalCategories,
  listLocalCollections,
  listLocalPrompts,
  recordLocalPromptUse,
  setLocalSetting,
  updateLocalPrompt,
} from "../platform/library.js";
import "../styles/workbench-chrome.css";

const props = defineProps({
  databaseStatus: { type: String, default: "pending" },
  localCount: { type: Number, default: 0 },
  host: { type: String, default: () => detectHost() },
});
const trafficInset = computed(() => trafficLightInsetPx(props.host));
const shortcutLabel = computed(() => formatShortcutLabel(DEFAULT_LAUNCHER_SHORTCUT, props.host));

const emit = defineEmits(["open-launcher", "library-changed"]);

const space = ref("local");
const selectedId = ref(null);
const dark = ref(false);
const view = ref("grid");
const sortTab = ref("全部");
const creating = ref(false);
const editing = ref(null);
const using = ref(null);
const settingsOpen = ref(false);
const openedCollection = ref(null);
const collectionMembers = ref([]);
const query = ref("");
const prompts = ref([]);
const collections = ref([]);
const categoryGroups = ref([]);
const addingCategoryId = ref("");
const newCategoryName = ref("");
const categoryError = ref("");
const theme = ref("light");
const session = ref(getSession());
const loginReason = ref("");
const publishResume = ref(false);
const pendingPublish = ref(false);
const publishSources = ref([]);
const publishSourceId = ref("");
const squareItems = ref([]);
const squareOffline = ref(false);
const squareBlocked = ref(false);
const favoriteIds = ref([]);
const libraryItems = computed(() => [
  ...collections.value.map((item) => ({ ...item, kind: "collection" })),
  ...prompts.value.map((item) => ({ ...item, kind: "prompt" })),
]);
const displayedItems = computed(() => (space.value === "square" ? squareItems.value : libraryItems.value));
const filterTabs = computed(() => (space.value === "square" ? ["推荐", "最新", "热门", "收藏"] : ["全部", "最近", "收藏"]));
const selectedLabel = computed(() => {
  if (!selectedId.value) return "全部提示词";
  for (const group of categoryGroups.value) {
    if (group.id === selectedId.value) return group.name;
    const child = group.children.find((item) => item.id === selectedId.value);
    if (child) return child.name;
  }
  return "全部提示词";
});

const locationLabel = computed(() => (space.value === "square" ? "提示词广场" : "本地提示词"));
const databaseLabel = computed(() => {
  if (props.databaseStatus === "ready") return "SQLite 就绪";
  if (props.databaseStatus === "failed") return "SQLite 失败";
  return "SQLite 未接入";
});

function toggleGroup(group) {
  group.open = !group.open;
  selectCategory(group.id);
}

function collapseAll() {
  for (const group of categoryGroups.value) group.open = false;
}

function selectCategory(id) {
  selectedId.value = id;
  reloadPrompts();
}

function categoryById(id) {
  for (const group of categoryGroups.value) {
    if (group.id === id) return group;
    const child = group.children.find((item) => item.id === id);
    if (child) return child;
  }
  return null;
}

function startAddCategory() {
  categoryError.value = "";
  addingCategoryId.value = "";
  const current = categoryById(selectedId.value);
  if (!current) {
    categoryError.value = "请先选中一个大分类";
    return;
  }
  if (current.parent_id) {
    categoryError.value = "小分类下不能再创建子分类";
    return;
  }
  addingCategoryId.value = current.id;
  newCategoryName.value = "";
}

async function confirmAddCategory() {
  categoryError.value = "";
  try {
    await createLocalCategory({ name: newCategoryName.value, parentId: addingCategoryId.value });
    addingCategoryId.value = "";
    newCategoryName.value = "";
    categoryGroups.value = buildCategoryTree(await listLocalCategories());
    const parent = categoryGroups.value.find((group) => group.id === selectedId.value);
    if (parent) parent.open = true;
  } catch (error) {
    categoryError.value = error instanceof Error ? error.message : String(error);
  }
}

function openLogin(reason) {
  loginReason.value = reason;
}

async function logoutFromSettings() {
  await logoutSession();
  session.value = getSession();
}

async function downloadSquare(item) {
  try {
    await downloadSquareItem(item.id);
  } catch {
    squareOffline.value = true;
  }
}

async function favoriteSquare(item) {
  if (!getSession().loggedIn) {
    openLogin("收藏需要登录");
    return;
  }
  try {
    if (favoriteIds.value.includes(item.id)) {
      await deleteFavorite(item.id);
      favoriteIds.value = favoriteIds.value.filter((id) => id !== item.id);
    } else {
      await putFavorite(item.id);
      favoriteIds.value = [...favoriteIds.value, item.id];
    }
  } catch {
    squareOffline.value = true;
  }
}

async function loadPublishSources() {
  const [localPrompts, localCollections] = await Promise.all([
    listLocalPrompts({ query: "", categoryId: null }),
    listLocalCollections({ query: "", categoryId: null }),
  ]);
  publishSources.value = [
    ...localPrompts.map((item) => ({ id: item.id, title: item.title, kind: "prompt" })),
    ...localCollections.map((item) => ({ id: item.id, title: item.title, kind: "collection" })),
  ];
  publishSourceId.value = "";
}

async function openPublish() {
  await loadPublishSources();
  publishResume.value = true;
}

async function startPublish() {
  if (!getSession().loggedIn) {
    pendingPublish.value = true;
    openLogin("发布需要登录");
    return;
  }
  await openPublish();
}

async function finishLogin() {
  session.value = getSession();
  loginReason.value = "";
  await refreshFavorites();
  if (pendingPublish.value) {
    pendingPublish.value = false;
    await openPublish();
  }
}

async function submitPublish() {
  if (!publishSourceId.value) return;
  try {
    await createPublication({ sourceId: publishSourceId.value });
    publishResume.value = false;
  } catch {
    squareOffline.value = true;
  }
}

async function refreshFavorites() {
  if (!getSession().loggedIn) {
    favoriteIds.value = [];
    return;
  }
  try {
    const rows = await listFavorites();
    favoriteIds.value = rows.map((row) => row.id);
  } catch {
    favoriteIds.value = [];
  }
}

async function loadSquare() {
  squareOffline.value = false;
  squareBlocked.value = false;
  const access = await getLocalSetting("square_access");
  if (access === "0") {
    squareItems.value = [];
    squareBlocked.value = true;
    return;
  }
  try {
    if (sortTab.value === "收藏") {
      if (!getSession().loggedIn) {
        squareItems.value = [];
        openLogin("收藏需要登录");
        return;
      }
      squareItems.value = await listFavorites();
    } else {
      squareItems.value = await listSquareItems({ sort: sortTab.value, query: query.value });
    }
  } catch {
    squareItems.value = [];
    squareOffline.value = true;
  }
}

function setSort(tab) {
  sortTab.value = tab;
  if (space.value === "square") loadSquare();
}

function openSquare() {
  space.value = "square";
  sortTab.value = "推荐";
  loadSquare();
}

function openLocal() {
  space.value = "local";
  sortTab.value = "全部";
  squareOffline.value = false;
  reloadPrompts();
}

function toggleTheme() {
  applyTheme(theme.value === "dark" ? "light" : "dark");
}

async function applyTheme(next) {
  theme.value = next;
  const prefersDark =
    typeof window !== "undefined" && Boolean(window.matchMedia?.("(prefers-color-scheme: dark)")?.matches);
  dark.value = next === "dark" || (next === "system" && prefersDark);
  document.body.classList.toggle("theme-dark", dark.value);
  await setLocalSetting("theme", next);
}

async function reloadPrompts() {
  if (space.value !== "local") return;
  const filter = { query: query.value, categoryId: selectedId.value };
  const [filteredPrompts, filteredCollections, allPrompts] = await Promise.all([
    listLocalPrompts(filter),
    listLocalCollections(filter),
    listLocalPrompts({ query: "", categoryId: null }),
  ]);
  prompts.value = filteredPrompts;
  collections.value = filteredCollections;
  emit("library-changed", allPrompts.length);
}

function closeEditor() {
  creating.value = false;
  editing.value = null;
}

function coverPreview(item) {
  return parseCoverUrls(item.cover_json).slice(0, 3);
}

async function savePrompt({ id, kind, title, content, categoryId, coverType, coverUrls }) {
  if (id) {
    await updateLocalPrompt({ id, title, content, categoryId });
  } else if (kind === "collection") {
    await createLocalCollection({ title, categoryId, coverType, coverUrls });
  } else {
    await createLocalPrompt({ title, content, categoryId });
  }
  closeEditor();
  query.value = "";
  await reloadPrompts();
}

async function removePrompt(id) {
  await deleteLocalPrompt(id);
  closeEditor();
  await reloadPrompts();
}

async function finishUse(text) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    /* browser may deny clipboard in tests */
  }
  await recordLocalPromptUse(using.value.id);
  using.value = null;
  await reloadPrompts();
}

function openItem(item) {
  if (space.value === "square") return;
  if (item.kind === "collection") {
    openCollection(item);
    return;
  }
  editing.value = item;
}

async function openCollection(collection) {
  openedCollection.value = collection;
  collectionMembers.value = await listCollectionMembers(collection.id);
}

async function addToOpenedCollection(promptId) {
  await addPromptToCollection(promptId, openedCollection.value.id);
  collectionMembers.value = await listCollectionMembers(openedCollection.value.id);
  await reloadPrompts();
}

onMounted(async () => {
  applyHostChrome(document.body, props.host);
  const stored = await getLocalSetting("theme");
  if (stored === "dark" || stored === "light" || stored === "system") {
    await applyTheme(stored);
  }
  categoryGroups.value = buildCategoryTree(await listLocalCategories());
  await reloadPrompts();
  await refreshFavorites();
  if (window.__TAURI_INTERNALS__) {
    const { listen } = await import("@tauri-apps/api/event");
    await listen("open-new-prompt", () => {
      creating.value = true;
    });
  }
});
</script>
