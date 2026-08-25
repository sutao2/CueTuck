<template>
  <div class="web-shell" data-testid="web-shell">
    <header data-region="titlebar" class="titlebar">
      <button
        type="button"
        class="sidebar-toggle"
        data-testid="toggle-sidebar"
        :aria-expanded="!sidebarCollapsed"
        @click="sidebarCollapsed = !sidebarCollapsed"
      >
        {{ sidebarCollapsed ? "打开侧栏" : "收起侧栏" }}
      </button>
      <span class="brand">提示方舟</span>
      <span class="kicker">浏览器工作台</span>
    </header>
    <div class="workspace">
      <aside
        data-region="sidebar"
        data-testid="sidebar"
        class="sidebar"
        :class="{ 'is-collapsed': sidebarCollapsed }"
      >
        <button type="button" class="space-tab" data-space="local" :class="{ active: space === 'local' }" @click="space = 'local'">
          本地提示词
        </button>
        <button type="button" class="space-tab" data-space="square" :class="{ active: space === 'square' }" @click="openSquare">
          提示词广场
        </button>
      </aside>
      <main data-region="content" class="content">
        <p data-testid="library-note" class="library-note">
          浏览器使用内存库，尚未与桌面 SQLite 同步。云同步未接通，不会假装已同步。
        </p>
        <section class="content-head">
          <h1>{{ space === "local" ? "本地提示词" : "提示词广场" }}</h1>
          <button
            v-if="space === 'local'"
            type="button"
            class="primary-button"
            data-testid="new-prompt"
            @click="startCreate"
          >
            新建
          </button>
        </section>
        <form v-if="space === 'local' && editing" class="editor" @submit.prevent="savePrompt">
          <input
            v-model="draftTitle"
            data-testid="prompt-title"
            class="title-input"
            placeholder="标题"
            autocomplete="off"
          >
          <textarea
            v-model="draftContent"
            data-testid="prompt-content"
            class="body-input"
            placeholder="正文。{{变量}} 会在使用时填写。"
          />
          <button type="button" class="primary-button" data-testid="save-prompt" @click="savePrompt">保存</button>
        </form>
        <ul v-if="space === 'local' && prompts.length" data-testid="prompt-list" class="prompt-list">
          <li v-for="row in prompts" :key="row.id">
            <button type="button" class="prompt-row" data-testid="prompt-row" @click="openPrompt(row.id)">
              {{ row.title }}
            </button>
          </li>
        </ul>
        <article v-if="space === 'local' && opened && !editing && !using" class="prompt-detail">
          <h2>{{ opened.title }}</h2>
          <pre data-testid="prompt-body" class="prompt-body">{{ opened.content }}</pre>
          <button type="button" class="primary-button" data-testid="edit-prompt" @click="startEdit">编辑</button>
          <button type="button" class="primary-button" data-testid="use-prompt" @click="startUse">使用</button>
        </article>
        <section v-if="space === 'local' && using" class="wizard" data-testid="use-wizard">
          <div v-if="wizardStep === 'fill'">
            <p data-testid="wizard-step">{{ wizardNames[wizardIndex] }}</p>
            <input
              v-model="draftVar"
              data-testid="wizard-var"
              class="title-input"
              @keydown.enter.prevent="wizardNext"
            >
            <button type="button" class="primary-button" data-testid="wizard-next" @click="wizardNext">下一步</button>
          </div>
          <div v-else>
            <pre data-testid="wizard-preview" class="prompt-body">{{ previewText }}</pre>
            <button type="button" class="primary-button" data-testid="wizard-copy" @click="copyPreview">复制</button>
          </div>
        </section>
        <p v-if="space === 'local' && !prompts.length" class="empty">浏览器内存库是空的。点「新建」只会写在这个标签页里。</p>
        <div v-else-if="space === 'square'" class="square-pane">
          <p v-if="squareOffline" data-testid="square-offline" class="empty">
            当前离线。预发广场暂时不可用，本地内存库仍可使用。
          </p>
          <button v-if="squareOffline" type="button" class="primary-button" data-testid="go-local" @click="openLocal">前往本地</button>
          <p v-if="favoriteNote" data-testid="favorite-note">{{ favoriteNote }}</p>
          <section v-if="loginOpen" data-testid="login-modal" class="editor">
            <label>
              <span>邮箱</span>
              <input v-model="loginEmail" type="email" data-testid="login-email" autocomplete="username">
            </label>
            <label>
              <span>密码</span>
              <input v-model="loginPassword" type="password" data-testid="login-password" autocomplete="current-password">
            </label>
            <p v-if="loginError" data-testid="login-error">{{ loginError }}</p>
            <button type="button" class="primary-button" data-testid="login-submit" :disabled="loginBusy" @click="submitLogin">
              登录
            </button>
            <button
              v-for="name in oauthProviders"
              :key="name"
              type="button"
              :data-testid="`oauth-${name}`"
              :disabled="loginBusy"
              @click="submitOAuth(name)"
            >
              {{ name === "google" ? "Google 登录" : "GitHub 登录" }}
            </button>
          </section>
          <ul v-if="!squareOffline && squareItems.length" data-testid="square-list" class="prompt-list">
            <li v-for="item in squareItems" :key="item.id">
              <span>{{ item.title }}</span>
              <button type="button" data-testid="square-download" @click="downloadItem(item.id)">下载</button>
              <button type="button" data-testid="square-favorite" @click="favoriteItem(item.id)">收藏</button>
            </li>
          </ul>
          <p v-else-if="!squareOffline" class="empty">广场仍走本仓库预发 API。未开后端时列表为空。</p>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from "vue";
import { createLocalPrompt, getLocalPrompt, listLocalPrompts, updateLocalPrompt } from "./memoryLibrary.js";
import { extractVariables, renderPrompt } from "./renderPrompt.js";
import { downloadSquareItem, listSquareItems, putFavorite } from "./square.js";
import {
  getSession,
  listOAuthProviders,
  loginOAuthSession,
  loginSession,
} from "./session.js";

const sidebarCollapsed = ref(false);
const space = ref("local");
const prompts = ref([]);
const editing = ref(false);
const editingId = ref(null);
const draftTitle = ref("");
const draftContent = ref("");
const opened = ref(null);
const using = ref(false);
const wizardNames = ref([]);
const wizardIndex = ref(0);
const wizardValues = ref({});
const wizardStep = ref("fill");
const draftVar = ref("");
const squareItems = ref([]);
const squareOffline = ref(false);
const favoriteNote = ref("");
const loginOpen = ref(false);
const loginEmail = ref("");
const loginPassword = ref("");
const loginError = ref("");
const loginBusy = ref(false);
const oauthProviders = ref([]);
const pendingFavorite = ref("");
let loginAbort = new AbortController();

function reload() {
  prompts.value = listLocalPrompts();
}

function startCreate() {
  editing.value = true;
  editingId.value = null;
  opened.value = null;
  using.value = false;
  draftTitle.value = "";
  draftContent.value = "";
}

function startEdit() {
  if (!opened.value) return;
  editing.value = true;
  editingId.value = opened.value.id;
  draftTitle.value = opened.value.title;
  draftContent.value = opened.value.content;
}

function openPrompt(id) {
  editing.value = false;
  editingId.value = null;
  using.value = false;
  opened.value = getLocalPrompt(id);
}

function startUse() {
  if (!opened.value) return;
  editing.value = false;
  using.value = true;
  wizardNames.value = extractVariables(opened.value.content);
  wizardIndex.value = 0;
  wizardValues.value = {};
  draftVar.value = "";
  wizardStep.value = wizardNames.value.length ? "fill" : "preview";
}

function wizardNext() {
  const name = wizardNames.value[wizardIndex.value];
  if (name) wizardValues.value[name] = draftVar.value;
  if (wizardIndex.value < wizardNames.value.length - 1) {
    wizardIndex.value += 1;
    draftVar.value = wizardValues.value[wizardNames.value[wizardIndex.value]] ?? "";
    return;
  }
  wizardStep.value = "preview";
}

const previewText = computed(() => renderPrompt(opened.value?.content ?? "", wizardValues.value));

async function copyPreview() {
  await navigator.clipboard.writeText(previewText.value);
}

function savePrompt() {
  if (!draftTitle.value.trim()) return;
  if (editingId.value) {
    updateLocalPrompt({
      id: editingId.value,
      title: draftTitle.value,
      content: draftContent.value,
    });
  } else {
    createLocalPrompt({ title: draftTitle.value, content: draftContent.value });
  }
  const keepId = editingId.value;
  editing.value = false;
  editingId.value = null;
  draftTitle.value = "";
  draftContent.value = "";
  reload();
  opened.value = keepId ? getLocalPrompt(keepId) : (prompts.value[0] ?? null);
}

function openLocal() {
  space.value = "local";
}

async function openSquare() {
  space.value = "square";
  favoriteNote.value = "";
  loginOpen.value = false;
  squareOffline.value = false;
  try {
    squareItems.value = await listSquareItems();
  } catch {
    squareOffline.value = true;
    squareItems.value = [];
  }
}

async function downloadItem(id) {
  await downloadSquareItem(id);
  reload();
  space.value = "local";
}

async function favoriteItem(id) {
  if (!getSession().loggedIn) {
    favoriteNote.value = "收藏需要登录";
    pendingFavorite.value = id;
    loginOpen.value = true;
    loginError.value = "";
    await loadProviders();
    return;
  }
  try {
    await putFavorite(id, getSession().accessToken);
    favoriteNote.value = "已收藏";
  } catch {
    favoriteNote.value = "收藏失败";
  }
}

async function loadProviders() {
  try {
    const payload = await listOAuthProviders();
    oauthProviders.value = (payload.items ?? []).filter(
      (name) => name === "google" || name === "github",
    );
  } catch {
    oauthProviders.value = [];
  }
}

async function afterLogin() {
  loginOpen.value = false;
  loginBusy.value = false;
  const id = pendingFavorite.value;
  pendingFavorite.value = "";
  if (id) await favoriteItem(id);
}

async function submitLogin() {
  if (loginBusy.value) return;
  loginError.value = "";
  try {
    await loginSession({ email: loginEmail.value, password: loginPassword.value });
    await afterLogin();
  } catch (caught) {
    loginError.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function submitOAuth(provider) {
  if (loginBusy.value) return;
  loginError.value = "";
  loginBusy.value = true;
  loginAbort.abort();
  loginAbort = new AbortController();
  try {
    await loginOAuthSession(provider, { signal: loginAbort.signal });
    await afterLogin();
  } catch (caught) {
    if (loginAbort.signal.aborted) return;
    loginError.value = caught instanceof Error ? caught.message : String(caught);
    loginBusy.value = false;
  }
}

onMounted(reload);
</script>
