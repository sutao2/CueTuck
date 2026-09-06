<template>
  <div class="app-shell" :class="{ 'sidebar-collapsed': sidebarCollapsed }">
    <header
      data-region="titlebar"
      class="titlebar"
      :class="{ 'host-mac': host === 'macos' }"
      :style="{ '--traffic-light-inset': `${trafficInset}px` }"
      data-tauri-drag-region
    >
      <div class="titlebar-left" data-tauri-drag-region>
        <button type="button" class="app-mark" :aria-label="t('brand')">P</button>
        <span class="brand-name">{{ t("brand") }}</span>
        <button type="button" class="sidebar-toggle" data-testid="toggle-sidebar" :aria-label="sidebarCollapsed ? '展开侧栏' : '收起侧栏'" :aria-expanded="!sidebarCollapsed" aria-controls="workbench-sidebar" @click="sidebarCollapsed = !sidebarCollapsed"><AppIcon name="teal" /></button>
      </div>
      <div class="titlebar-center" data-tauri-drag-region>
        <span>{{ locationLabel }}</span>
      </div>
      <div class="titlebar-right">
        <button type="button" class="title-tool" :title="t('search')" @click="$emit('open-launcher')">
          <AppIcon name="search" /><span>{{ t("search") }}</span><kbd>{{ shortcutLabel }}</kbd>
        </button>
      </div>
    </header>

    <div class="workspace">
      <aside v-show="!sidebarCollapsed" id="workbench-sidebar" data-region="sidebar" class="sidebar">
        <div class="space-switch" role="tablist" aria-label="提示词空间">
          <button
            type="button"
            class="space-tab"
            data-space="square"
            role="tab"
            :aria-selected="space === 'square'"
            :class="{ active: space === 'square' }"
            @click="openSquare"
          >
            <span class="nav-icon"><AppIcon name="square" /></span><span>{{ t("square") }}</span>
          </button>
          <button
            type="button"
            class="space-tab"
            data-space="local"
            role="tab"
            :aria-selected="space === 'local'"
            :class="{ active: space === 'local' }"
            @click="openLocal"
          >
            <span class="nav-icon"><AppIcon name="library" /></span><span>{{ t("local") }}</span>
            <span class="nav-count">{{ localCount }}</span>
          </button>
        </div>

        <div class="sidebar-toolbar">
          <span>{{ space === "local" ? t("myCategories") : t("exploreCategories") }}</span>
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
            <span class="tree-icon warm"><AppIcon name="square" /></span>
            <span>{{ t("allPrompts") }}</span>
            <span v-if="space === 'local'" class="tree-count">{{ allLocalItems.length }}</span>
          </button>
          <button v-if="space === 'local'" type="button" class="tree-row" data-testid="uncategorized"
            :class="{ active: selectedId === '__uncategorized__' }" @click="selectCategory('__uncategorized__')">
            <span class="chevron ghost">›</span>
            <span class="tree-icon"><AppIcon name="folder" /></span>
            <span>{{ uiLanguage === 'en' ? 'Uncategorized' : '未分类' }}</span>
            <span class="tree-count">{{ categoryCount('__uncategorized__') }}</span>
          </button>
          <div v-for="group in categoryGroups" :key="group.id" class="tree-group" :class="{ open: group.open }">
            <button type="button" class="tree-row tree-parent" :class="{ active: selectedId === group.id }" @click="toggleGroup(group)">
              <span class="chevron">›</span>
              <span class="tree-icon" :class="group.tone"><AppIcon :name="group.tone" /></span>
              <span>{{ group.name }}</span>
              <span v-if="space === 'local'" class="tree-count">{{ categoryCount(group.id) }}</span>
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
                <span v-if="space === 'local'" class="tree-count">{{ categoryCount(child.id) }}</span>
              </button>
            </div>
          </div>
        </nav>

        <div class="sidebar-bottom">
          <button type="button" data-testid="open-settings" @click="settingsOpen = true">
            <AppIcon name="settings" /><span>{{ t("settings") }}</span><span class="sidebar-bottom-action">›</span>
          </button>
          <div class="sidebar-account">
            <button type="button" class="account-button" data-testid="open-login"
              :title="session.loggedIn ? session.email : t('login')" @click="openLogin(t('login'))">
              <span class="avatar">{{ session.loggedIn ? (session.email?.[0] || "已") : "游" }}</span>
              <span>{{ session.loggedIn ? t("loggedIn") : t("login") }}</span>
            </button>
            <button type="button" class="preference-toggle" :title="dark ? t('switchLight') : t('switchDark')" @click="toggleTheme">
              <AppIcon :name="dark ? 'sun' : 'moon'" />
            </button>
            <button type="button" class="preference-toggle language-toggle" :title="t('languageToggle')" @click="toggleLanguage">
              {{ uiLanguage === "en" ? "中" : "EN" }}
            </button>
          </div>
        </div>
      </aside>

      <main data-region="content" class="content-area">
        <section class="content-header">
          <div class="content-heading">
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
            <button v-if="space === 'square'" type="button" class="button ghost-button" @click="loadSquare"><AppIcon name="refresh" />{{ t("refresh") }}</button>
            <button
              v-if="space === 'square'"
              type="button"
              class="button primary-button"
              data-testid="publish-prompt"
              @click="startPublish"
            >
              <AppIcon name="plus" /><span>{{ t("publish") }}</span>
            </button>
            <button
              v-else
              type="button"
              class="button primary-button"
              @click="creating = true"
            >
              <AppIcon name="plus" /><span>{{ t("create") }}</span>
            </button>
          </div>
        </section>

        <section class="filter-bar">
          <label class="inline-search">
            <AppIcon name="search" />
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
              :key="tab.id"
              type="button"
              :data-sort="tab.id"
              :class="{ active: sortTab === tab.id }"
              @click="setSort(tab.id)"
            >
              {{ tab.label }} <small>{{ tabCount(tab.id) }}</small>
            </button>
          </div>
          <div class="filter-spacer"></div>
          <label class="compact-select">
            <span>{{ t("model") }}</span>
            <select data-testid="model-filter" v-model="modelFilter" @change="onModelFilter">
              <option value="">{{ t("allModels") }}</option>
              <option v-for="name in modelOptions" :key="name" :value="name">{{ name }}</option>
            </select>
          </label>
          <div class="view-switch" aria-label="视图切换">
            <button type="button" :class="{ active: view === 'grid' }" title="网格视图" @click="view = 'grid'"><AppIcon name="grid" /></button>
            <button type="button" :class="{ active: view === 'list' }" title="列表视图" @click="view = 'list'"><AppIcon name="list" /></button>
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

        <p v-if="operationNote" role="status" data-testid="operation-note" class="use-hint">{{ operationNote }}</p>
        <section class="prompt-section">
          <div class="section-heading-row">
            <div>
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
              @contextmenu.prevent="openContextMenu($event, item)"
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
                <span v-if="showModelTags && item.model" class="model-tag" data-testid="model-tag">{{ item.model }}</span>
              </div>
              <h3>{{ item.title }}</h3>
              <p v-if="item.author" class="prompt-author" data-testid="prompt-author">{{ item.author }}</p>
              <p class="prompt-excerpt">
                {{
                  item.kind === "collection"
                    ? `${item.member_count ?? 0} 个提示词`
                    : item.content || item.excerpt || "还没有正文"
                }}
              </p>
              <div v-if="item.kind === 'prompt' || space === 'square'" class="card-footer">
                <template v-if="space === 'square'">
                  <button
                    type="button"
                    class="card-action"
                    data-testid="download-square"
                    :disabled="downloadBusy.includes(item.id)"
                    @click.stop="downloadSquare(item)"
                  >
                    下载
                  </button>
                  <button
                    type="button"
                    class="card-action"
                    data-testid="favorite-square"
                    :disabled="favoriteBusy.includes(item.id)"
                    @click.stop="favoriteSquare(item)"
                  >
                    {{ favoriteIds.includes(item.id) ? "已收藏" : "收藏" }}
                  </button>
                </template>
                <button v-else type="button" class="card-action" @click.stop="startUse(item)">使用</button>
              </div>
            </article>
          </div>
          <div v-else class="empty-state">
            <span class="empty-glyph"><AppIcon :name="space === 'square' ? 'square' : 'library'" /></span>
            <h3>{{ emptyHeading }}</h3>
            <p>{{ emptyCopy }}</p>
          </div>
        </section>
      </main>
    </div>

    <CreatePromptModal
      v-if="creating || editing"
      :prompt="editing"
      :groups="categoryGroups"
      :model-options="modelOptions"
      :default-model="defaultModel"
      :default-category-id="selectedId === '__uncategorized__' ? '' : (selectedId || '')"
      :error="editorError"
      :busy="editorBusy"
      @cancel="closeEditor"
      @save="savePrompt"
      @remove="removePrompt"
    />
    <UsePromptModal
      v-if="using"
      :prompt="using"
      :hints-enabled="variableHints"
      :error="useError"
      :busy="useBusy"
      @cancel="using = null"
      @copied="finishUse"
    />
    <CollectionDetailModal
      v-if="openedCollection"
      :collection="openedCollection"
      :members="collectionMembers"
      :prompts="collectionCandidates"
      :error="collectionError"
      @cancel="openedCollection = null"
      @add="addToOpenedCollection"
      @remove-member="removeFromOpenedCollection"
      @open="openCollectionMember"
      @use="useCollectionMember"
      @edit="editOpenedCollection"
    />
    <SettingsModal
      v-if="settingsOpen"
      :theme="theme"
      :host="host"
      :session="session"
      :language="uiLanguage"
      @cancel="closeSettings"
      @language="applyUiLanguage"
      @theme="applyTheme($event, false)"
      @imported="refreshLocalSettings"
      @history-cleared="reloadPrompts"
      @login="openLogin('登录账号')"
      @logout="logoutFromSettings"
    />
    <SquareDetailModal
      v-if="squareDetail"
      :item="squareDetail"
      :loading="squareDetailLoading"
      :error="squareDetailError"
      :note="operationNote"
      :downloading="downloadBusy.includes(squareDetail.id)"
      :favorite="favoriteIds.includes(squareDetail.id)"
      :favorite-busy="favoriteBusy.includes(squareDetail.id)"
      @cancel="closeSquareDetail"
      @retry="openSquareDetail(squareDetail)"
      @download="downloadSquare(squareDetail)"
      @favorite="favoriteSquare(squareDetail)"
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
          <p v-if="operationNote" role="status" class="use-hint">{{ operationNote }}</p>
          <p>提交后本地正文仍可编辑，审核状态不会覆盖本机内容。</p>
        </div>
        <footer class="modal-footer">
          <button type="button" class="button ghost-button" @click="publishResume = false">关闭</button>
          <button
            type="button"
            class="button primary-button"
            data-testid="publish-submit"
            :disabled="!publishSourceId || publishBusy"
            @click="submitPublish"
          >
            提交审核
          </button>
        </footer>
      </section>
    </div>

    <div
      v-if="contextMenu"
      class="context-menu-layer"
      data-testid="context-menu-layer"
      @click="contextMenu = null"
      @contextmenu.prevent="contextMenu = null"
    >
      <div
        class="context-menu"
        data-testid="context-menu"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
        @click.stop
      >
        <button
          v-for="action in contextActions(contextMenu.item)"
          :key="action.id"
          type="button"
          :data-action="action.id"
          @click="runContextAction(action.id)"
        >
          {{ action.label }}
        </button>
      </div>
    </div>
    <footer data-region="statusbar" class="statusbar">
      <span class="status-item">
        <span class="connection-dot" :class="databaseStatus === 'ready' ? 'online' : 'offline'"></span>
        <span>{{ t("localFirst") }}</span>
      </span>
      <span class="status-sep"></span>
      <span class="status-item">{{ databaseLabel }}</span>
      <span class="status-item">本地 <strong>{{ localCount }}</strong> 条</span>
      <span class="status-spacer"></span>
      <span class="status-item muted-status">{{ t("moreActions") }}</span>
      <span class="status-sep"></span>
      <button type="button" class="status-button" @click="$emit('open-launcher')">
        <AppIcon name="search" /> 快捷搜索 <kbd>{{ shortcutLabel }}</kbd>
      </button>
    </footer>
  </div>
</template>

<script setup>
import AppIcon from "./AppIcon.vue";
import { computed, onMounted, onUnmounted, ref } from "vue";
import CollectionDetailModal from "./CollectionDetailModal.vue";
import CreatePromptModal from "./CreatePromptModal.vue";
import LoginModal from "./LoginModal.vue";
import SettingsModal from "./SettingsModal.vue";
import SquareDetailModal from "./SquareDetailModal.vue";
import UsePromptModal from "./UsePromptModal.vue";
import { getSession, logoutSession } from "../platform/session.js";
import { filterLocalItems, listLocalFavoriteIds, toggleLocalFavorite } from "../platform/localFavorites.js";
import { parseModelNames } from "../platform/modelCatalog.js";
import { uiText } from "../platform/uiStrings.js";
import { downloadSquareItem, fetchSquareContent, listFavorites, listSquareItems } from "../platform/square.js";
import { applyQueuedFavorites, favoriteWithQueue, publishWithQueue } from "../platform/syncQueue.js";
import { parseCoverUrls } from "../lib/cover.js";
import { DEFAULT_LAUNCHER_SHORTCUT } from "../platform/shortcut.js";
import { applyHostChrome, detectHost, formatShortcutLabel, trafficLightInsetPx } from "../platform/windowChrome.js";
import {
  addPromptToCollection,
  removePromptFromCollection,
  updateLocalCollection,
  deleteLocalCollection,
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
const sidebarCollapsed = ref(false);

function handleWorkbenchShortcut(event) {
  const modifier = props.host === 'macos' ? event.metaKey : event.ctrlKey;
  if (!modifier || event.altKey || event.shiftKey || event.repeat) return;
  if (creating.value || editing.value || using.value || openedCollection.value || loginReason.value || pendingPublish.value || squareDetail.value) return;
  if (event.key === ',') { event.preventDefault(); settingsOpen.value = true; return; }
  const target = event.target;
  if (event.key.toLowerCase() === 'b' && !settingsOpen.value && !target?.closest?.('input, textarea, select, [contenteditable="true"]')) {
    event.preventDefault(); sidebarCollapsed.value = !sidebarCollapsed.value;
  }
}
onMounted(() => window.addEventListener('keydown', handleWorkbenchShortcut));
onUnmounted(() => window.removeEventListener('keydown', handleWorkbenchShortcut));
const selectedId = ref(null);
const dark = ref(false);
const view = ref("grid");
const sortTab = ref("全部");
const creating = ref(false);
const editing = ref(null);
const using = ref(null);
const useError = ref("");
const useBusy = ref(false);
const editorError = ref("");
const editorBusy = ref(false);
const collectionError = ref("");
const collectionCandidates = ref([]);
const settingsOpen = ref(false);
const openedCollection = ref(null);
const collectionMembers = ref([]);
const query = ref("");
const prompts = ref([]);
const collections = ref([]);
const allLocalItems = ref([]);
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
const publishBusy = ref(false);
const favoriteBusy = ref([]);
const operationNote = ref("");
const squareItems = ref([]);
const squareOffline = ref(false);
const squareBlocked = ref(false);
const favoriteIds = ref([]);
const localFavoriteIds = ref([]);
const contextMenu = ref(null);
const modelFilter = ref("");
const modelCatalogText = ref("");
const customModelsText = ref("");
const seenModels = ref([]);
const defaultModel = ref("");
const showModelTags = ref(true);
const variableHints = ref(false);
const uiLanguage = ref("zh");

function t(key) {
  return uiText(uiLanguage.value, key);
}
const libraryItems = computed(() => [
  ...collections.value.map((item) => ({ ...item, kind: "collection" })),
  ...prompts.value.map((item) => ({ ...item, kind: "prompt" })),
]);
const displayedItems = computed(() => {
  const rows =
    space.value === "square"
      ? squareItems.value
      : filterLocalItems(libraryItems.value, {
          tab: sortTab.value,
          favoriteIds: localFavoriteIds.value,
        });
  if (!modelFilter.value) return rows;
  return rows.filter((item) => item.kind === "prompt" && item.model === modelFilter.value);
});
const filterTabs = computed(() =>
  space.value === "square"
    ? [
        { id: "推荐", label: t("tabRecommended") },
        { id: "最新", label: t("tabLatest") },
        { id: "热门", label: t("tabHot") },
        { id: "收藏", label: t("tabFavorite") },
      ]
    : [
        { id: "全部", label: t("tabAll") },
        { id: "最近", label: t("tabRecent") },
        { id: "收藏", label: t("tabFavorite") },
      ],
);
const selectedLabel = computed(() => {
  if (!selectedId.value) return t("allPrompts");
  if (selectedId.value === "__uncategorized__") return uiLanguage.value === "en" ? "Uncategorized" : "未分类";
  for (const group of categoryGroups.value) {
    if (group.id === selectedId.value) return group.name;
    const child = group.children.find((item) => item.id === selectedId.value);
    if (child) return child.name;
  }
  return t("allPrompts");
});

const modelOptions = computed(() =>
  parseModelNames(modelCatalogText.value, customModelsText.value, seenModels.value, prompts.value),
);
const emptyHeading = computed(() => {
  if (space.value === "square") return squareOffline.value ? t("emptyOffline") : t("emptySquare");
  if (sortTab.value === "最近") return t("emptyRecent");
  if (sortTab.value === "收藏") return t("emptyFavorite");
  return t("emptyLocal");
});
const emptyCopy = computed(() => (space.value === "square" ? t("emptySquareHint") : t("emptyLocalHint")));
const locationLabel = computed(() => (space.value === "square" ? t("square") : t("local")));
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
  return space.value === "square" ? loadSquare() : reloadPrompts();
}

function categoryCount(id) {
  return allLocalItems.value.filter((item) => {
    const category = categoryById(item.category_id);
    if (id === "__uncategorized__") return !category;
    return item.category_id === id || category?.parent_id === id;
  }).length;
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

const squareDetail = ref(null);
const squareDetailLoading = ref(false);
const squareDetailError = ref("");
const downloadBusy = ref([]);
let detailRequest = 0;

function closeSquareDetail() {
  detailRequest += 1;
  squareDetail.value = null;
}

async function openSquareDetail(item) {
  const request = ++detailRequest;
  squareDetail.value = { ...item };
  squareDetailLoading.value = true;
  squareDetailError.value = "";
  operationNote.value = "";
  try {
    const content = await fetchSquareContent(item.id);
    if (request === detailRequest) squareDetail.value = { ...item, ...content };
  } catch (error) {
    if (request === detailRequest) squareDetailError.value = `读取详情失败：${error.message || error}`;
  } finally {
    if (request === detailRequest) squareDetailLoading.value = false;
  }
}

async function downloadSquare(item) {
  if (downloadBusy.value.includes(item.id)) return;
  downloadBusy.value = [...downloadBusy.value, item.id];
  try {
    await downloadSquareItem(item.id);
    operationNote.value = `「${item.title}」已下载到本地。`;
    await reloadPrompts();
  } catch (error) {
    operationNote.value = `下载失败：${error.message || error}`;
  } finally { downloadBusy.value = downloadBusy.value.filter((id) => id !== item.id); }
}

async function favoriteSquare(item) {
  if (favoriteBusy.value.includes(item.id)) return;
  if (!getSession().loggedIn) {
    openLogin("收藏需要登录");
    return;
  }
  const removing = favoriteIds.value.includes(item.id);
  const email = getSession().email;
  favoriteBusy.value = [...favoriteBusy.value, item.id];
  try {
    const result = await favoriteWithQueue(item.id, removing ? "DELETE" : "PUT");
    if (getSession().email !== email) return;
    favoriteIds.value = removing
      ? favoriteIds.value.filter((id) => id !== item.id)
      : [...favoriteIds.value, item.id];
    operationNote.value = result.queued ? "已保存到本机队列，尚未送达服务器。" : (removing ? "已取消收藏。" : "已收藏。");
    if (removing && sortTab.value === "收藏") squareItems.value = squareItems.value.filter((row) => row.id !== item.id);
  } catch (error) {
    operationNote.value = `收藏操作失败：${error.message || error}`;
  } finally { favoriteBusy.value = favoriteBusy.value.filter((id) => id !== item.id); }
}

async function loadPublishSources() {
  const [localPrompts, localCollections] = await Promise.all([
    listLocalPrompts({ query: "", categoryId: null }),
    listLocalCollections({ query: "", categoryId: null }),
  ]);
  publishSources.value = [
    ...localPrompts.map((item) => ({ ...item, kind: "prompt" })),
    ...localCollections.map((item) => ({ ...item, kind: "collection" })),
  ];
  publishSourceId.value = "";
}

async function openPublish() {
  operationNote.value = "";
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
  if (!publishSourceId.value || publishBusy.value) return;
  publishBusy.value = true;
  const source = publishSources.value.find((item) => item.id === publishSourceId.value);
  try {
    if (!source) throw new Error("未选择本地内容");
    let members;
    if (source.kind === "collection") {
      members = (await listCollectionMembers(source.id)).map((member) => ({
        title: member.title, content: member.content, category_id: publicationCategory(member.category_id), model: member.model,
      }));
      if (!members.length) throw new Error("合集至少需要一条提示词才能发布");
      if (members.some((member) => !member.title?.trim() || !member.content?.trim())) throw new Error("合集成员标题和正文不能为空");
    }
    const result = await publishWithQueue({
      sourceId: publishSourceId.value,
      title: source?.title,
      content: source?.content ?? "",
      categoryId: publicationCategory(source?.category_id),
      model: source?.model,
      ...(source.kind === "collection" ? { kind: "collection", members } : {}),
    });
    publishResume.value = false;
    operationNote.value = result.queued ? "草稿已保存在本机队列，尚未提交审核。" : "已提交审核，本地内容仍可编辑。";
  } catch (error) {
    operationNote.value = `发布失败：${error.message || error}`;
  } finally { publishBusy.value = false; }
}

async function refreshFavorites() {
  if (!getSession().loggedIn) {
    favoriteIds.value = [];
    return;
  }
  try {
    const rows = await listFavorites();
    favoriteIds.value = await applyQueuedFavorites(rows.map((row) => row.id));
  } catch {
    favoriteIds.value = [];
  }
}

async function loadSquare() {
  const request = ++squareRequest;
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
      let rows = await listFavorites();
      if (request !== squareRequest || space.value !== "square") return;
      rows = rows.filter((item) => squareMatchesCategory(item)
        && (!query.value.trim() || item.title.toLowerCase().includes(query.value.trim().toLowerCase())));
      if (modelFilter.value) {
        rows = rows.filter((row) => row.model === modelFilter.value);
      }
      squareItems.value = rows;
    } else {
      const rows = await listSquareItems({
        sort: sortTab.value,
        query: query.value,
        model: modelFilter.value,
        categoryId: selectedId.value,
      });
      if (request !== squareRequest || space.value !== "square") return;
      squareItems.value = rows.filter(squareMatchesCategory);
    }
    rememberModels(squareItems.value);
  } catch {
    if (request !== squareRequest || space.value !== "square") return;
    squareItems.value = [];
    squareOffline.value = true;
  }
}

let squareRequest = 0;

function squareMatchesCategory(item) {
  return !selectedId.value || item.category_id === selectedId.value
    || categoryById(item.category_id)?.parent_id === selectedId.value;
}

function publicationCategory(id) {
  const category = categoryById(id);
  return category?.is_system ? category.id : category?.parent_id ?? null;
}

function rememberModels(items) {
  seenModels.value = parseModelNames(seenModels.value, items);
}

async function loadModelPrefs() {
  modelCatalogText.value = (await getLocalSetting("model_catalog")) || "";
  customModelsText.value = (await getLocalSetting("custom_models")) || "";
  defaultModel.value = (await getLocalSetting("default_model")) || "";
  showModelTags.value = (await getLocalSetting("show_model_tags")) !== "0";
  variableHints.value = (await getLocalSetting("variable_hints")) === "1";
  const storedLang = await getLocalSetting("ui_language");
  uiLanguage.value = storedLang === "en" ? "en" : "zh";
  document.documentElement.lang = uiLanguage.value === "en" ? "en" : "zh-CN";
  document.body.dataset.density = (await getLocalSetting('density')) === 'compact' ? 'compact' : 'comfortable';
}

async function applyUiLanguage(next) {
  uiLanguage.value = next === "en" ? "en" : "zh";
  document.documentElement.lang = uiLanguage.value === "en" ? "en" : "zh-CN";
  await setLocalSetting("ui_language", uiLanguage.value);
}

async function toggleLanguage() {
  await applyUiLanguage(uiLanguage.value === "en" ? "zh" : "en");
}

function onModelFilter() {
  if (space.value === "square") loadSquare();
}

function tabCount(tab) {
  if (space.value === "square") {
    return tab === sortTab.value ? displayedItems.value.length : 0;
  }
  return filterLocalItems(libraryItems.value, {
    tab,
    favoriteIds: localFavoriteIds.value,
  }).length;
}

function openContextMenu(event, item) {
  contextMenu.value = { x: event.clientX, y: event.clientY, item };
}

function contextActions(item) {
  if (space.value === "square") {
    return [
      { id: "download", label: t("download") },
      { id: "favorite", label: favoriteIds.value.includes(item.id) ? t("unfavorite") : t("favorite") },
    ];
  }
  if (item.kind === "collection") {
    return [{ id: "open", label: t("open") }];
  }
  return [
    { id: "edit", label: t("edit") },
    { id: "use", label: t("use") },
    {
      id: "favorite",
      label: localFavoriteIds.value.includes(item.id) ? t("unfavorite") : t("favorite"),
    },
    { id: "delete", label: t("remove") },
  ];
}

async function runContextAction(action) {
  const item = contextMenu.value?.item;
  contextMenu.value = null;
  if (!item) return;
  if (action === "edit" || action === "open") {
    openItem(item);
    return;
  }
  if (action === "use") {
    startUse(item);
    return;
  }
  if (action === "delete") {
    await removePrompt(item.id);
    return;
  }
  if (action === "download") {
    await downloadSquare(item);
    return;
  }
  if (action === "favorite") {
    if (space.value === "square") {
      await favoriteSquare(item);
      return;
    }
    localFavoriteIds.value = await toggleLocalFavorite(item.id);
  }
}

async function closeSettings() {
  settingsOpen.value = false;
  await refreshLocalSettings();
}

async function refreshLocalSettings() {
  categoryGroups.value = buildCategoryTree(await listLocalCategories());
  if (selectedId.value && selectedId.value !== "__uncategorized__" && !categoryById(selectedId.value)) selectedId.value = null;
  const storedTheme = (await getLocalSetting("theme")) || "light";
  if (["light", "dark", "system"].includes(storedTheme) && storedTheme !== theme.value) await applyTheme(storedTheme);
  await loadModelPrefs();
  await reloadPrompts();
}

function setSort(tab) {
  sortTab.value = tab;
  if (space.value === "square") loadSquare();
}

function openSquare() {
  space.value = "square";
  if (selectedId.value === "__uncategorized__" || (selectedId.value && !categoryById(selectedId.value)?.is_system)) {
    selectedId.value = null;
  }
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

async function applyTheme(next, persist = true) {
  if (persist) await setLocalSetting("theme", next);
  theme.value = next;
  const prefersDark =
    typeof window !== "undefined" && Boolean(window.matchMedia?.("(prefers-color-scheme: dark)")?.matches);
  dark.value = next === "dark" || (next === "system" && prefersDark);
  document.body.classList.toggle("theme-dark", dark.value);
}

async function reloadPrompts() {
  if (space.value !== "local") return;
  const request = ++localRequest;
  const filter = { query: query.value, categoryId: selectedId.value };
  const [filteredPrompts, filteredCollections, allPrompts, allCollections] = await Promise.all([
    listLocalPrompts(filter),
    listLocalCollections(filter),
    listLocalPrompts({ query: "", categoryId: null }),
    listLocalCollections({ query: "", categoryId: null }),
  ]);
  if (request !== localRequest || space.value !== "local") return;
  prompts.value = filteredPrompts;
  collections.value = filteredCollections;
  allLocalItems.value = [...allPrompts, ...allCollections];
  emit("library-changed", allPrompts.length);
}

let localRequest = 0;

function closeEditor() {
  creating.value = false;
  editing.value = null;
  editorError.value = "";
}

function coverPreview(item) {
  return parseCoverUrls(item.cover_json).slice(0, 3);
}

async function savePrompt({ id, kind, title, content, categoryId, model, coverType, coverUrls }) {
  if (editorBusy.value) return;
  editorBusy.value = true;
  editorError.value = "";
  try {
    if (id && kind === "collection") {
      await updateLocalCollection({ id, title, categoryId, coverType, coverUrls });
    } else if (id) {
      await updateLocalPrompt({ id, title, content, categoryId, model });
    } else if (kind === "collection") {
      await createLocalCollection({ title, categoryId, coverType, coverUrls });
    } else {
      await createLocalPrompt({ title, content, categoryId, model });
    }
    closeEditor();
    query.value = "";
    await reloadPrompts();
  } catch (error) {
    editorError.value = `保存失败：${error.message || error}`;
  } finally { editorBusy.value = false; }
}

async function removePrompt(id) {
  try {
    if (editing.value?.kind === "collection") await deleteLocalCollection(id);
    else await deleteLocalPrompt(id);
    closeEditor();
    await reloadPrompts();
  } catch (error) { editorError.value = `删除失败：${error.message || error}`; }
}

async function finishUse(text) {
  if (useBusy.value) return;
  const prompt = using.value;
  useBusy.value = true;
  useError.value = "";
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    useError.value = "复制失败，请检查剪贴板权限后重试；填写内容已保留。";
    useBusy.value = false;
    return;
  }
  try {
    await recordLocalPromptUse(prompt.id);
    if (using.value?.id === prompt.id) using.value = null;
    await reloadPrompts();
  } catch (error) { useError.value = `已复制，但保存使用记录失败：${error.message || error}`; }
  finally { useBusy.value = false; }
}

function openItem(item) {
  if (space.value === "square") {
    openSquareDetail(item);
    return;
  }
  if (item.kind === "collection") {
    openCollection(item);
    return;
  }
  editing.value = item;
}

async function openCollection(collection) {
  openedCollection.value = collection;
  collectionError.value = "";
  try {
    [collectionMembers.value, collectionCandidates.value] = await Promise.all([
      listCollectionMembers(collection.id), listLocalPrompts({ query: "", categoryId: null }),
    ]);
  } catch (error) { collectionError.value = `读取失败：${error.message || error}`; }
}

async function addToOpenedCollection(promptId) {
  try {
    await addPromptToCollection(promptId, openedCollection.value.id);
    await openCollection(openedCollection.value);
    await reloadPrompts();
  } catch (error) { collectionError.value = `加入失败：${error.message || error}`; }
}

async function removeFromOpenedCollection(promptId) {
  try {
    await removePromptFromCollection(promptId, openedCollection.value.id);
    await openCollection(openedCollection.value);
    await reloadPrompts();
  } catch (error) { collectionError.value = `移除失败：${error.message || error}`; }
}

function openCollectionMember(member) {
  openedCollection.value = null;
  editing.value = member;
}

function useCollectionMember(member) {
  openedCollection.value = null;
  startUse(member);
}

function startUse(member) {
  useError.value = "";
  using.value = member;
}

function editOpenedCollection() {
  editing.value = { ...openedCollection.value, kind: "collection" };
  openedCollection.value = null;
}

onMounted(async () => {
  applyHostChrome(document.body, props.host);
  const stored = await getLocalSetting("theme");
  if (stored === "dark" || stored === "light" || stored === "system") {
    await applyTheme(stored);
  }
  categoryGroups.value = buildCategoryTree(await listLocalCategories());
  await loadModelPrefs();
  localFavoriteIds.value = await listLocalFavoriteIds();
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
