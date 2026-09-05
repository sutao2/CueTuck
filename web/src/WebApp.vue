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
      <span v-if="session.loggedIn">{{ session.email }}</span>
      <button v-if="session.loggedIn" type="button" data-testid="logout" :disabled="syncBusy || downloadBusy.length > 0" @click="signOut">退出</button>
      <button v-else type="button" data-testid="open-login" @click="openLogin">登录</button>
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
          浏览器使用账号库或标签页内存库，尚未与桌面 SQLite 同步，不会写入本机 SQLite。
          未送达账号库的内容在刷新或关闭标签页后会丢失。
        </p>
        <p v-if="cloudNote" role="status" data-testid="cloud-note">{{ cloudNote }}</p>
        <button v-if="session.loggedIn" type="button" data-testid="reload-account" :disabled="syncBusy" @click="readAccount">重新读取账号库</button>
        <button v-if="session.loggedIn && prompts.some((row) => row.sync_pending)" type="button" data-testid="retry-save" :disabled="syncBusy" @click="retrySaves">重试保存到账号库</button>
        <section v-if="loginOpen" data-testid="login-modal" class="editor">
          <label><span>邮箱</span><input v-model="loginEmail" type="email" data-testid="login-email" autocomplete="username"></label>
          <label><span>密码</span><input v-model="loginPassword" type="password" data-testid="login-password" autocomplete="current-password"></label>
          <p v-if="loginError" data-testid="login-error">{{ loginError }}</p>
          <button type="button" class="primary-button" data-testid="login-submit" :disabled="loginBusy" @click="submitLogin">登录</button>
          <button v-for="name in oauthProviders" :key="name" type="button" :data-testid="`oauth-${name}`" :disabled="loginBusy" @click="submitOAuth(name)">{{ name === 'google' ? 'Google 登录' : 'GitHub 登录' }}</button>
          <button type="button" @click="cancelLogin">取消</button>
        </section>
        <section v-if="space === 'local'" class="billing" data-testid="billing">
          <p class="billing-head">
            <strong>账单</strong>
                <span data-testid="billing-pro">{{ session.loggedIn ? (billingPro ? "Pro" : "未订阅") : "未登录" }}</span>
                <small v-if="session.loggedIn && billingMock" data-testid="billing-mock">Mock · {{ billingMockPro ? "模拟 Pro" : "模拟未订阅" }}（不扣款，不改变真实权益）</small>
            <small v-if="billingNote" data-testid="billing-note">{{ billingNote }}</small>
            <span v-if="session.loggedIn && billingMock">
              <button v-for="(label, outcome) in { success: '模拟成功', failure: '模拟失败', cancel: '模拟取消', reset: '重置模拟' }" :key="outcome" type="button" :data-testid="`billing-mock-${outcome}`" :disabled="billingBusy" @click="runCheckout(outcome)">{{ label }}</button>
            </span>
          </p>
          <p class="billing-copy">预发可查状态、兑换码；只有测试密钥才跳转 Checkout。不是公开售卖。</p>
          <div class="billing-actions">
            <input
                  data-testid="billing-redeem-code"
                  :disabled="!session.loggedIn || billingMock || billingBusy"
              v-model="redeemCode"
              placeholder="兑换码"
            >
            <button
              type="button"
              class="primary-button"
                  data-testid="billing-redeem"
                  :disabled="!session.loggedIn || billingMock || billingBusy"
              @click="runRedeem"
            >
              兑换
            </button>
            <button
              type="button"
              class="primary-button"
                  data-testid="billing-checkout"
                  v-if="!billingMock"
                  :disabled="!session.loggedIn || billingBusy"
              @click="runCheckout"
            >
              前往支付
            </button>
          </div>
        </section>
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
          <button type="button" class="primary-button" data-testid="save-prompt" :disabled="syncBusy || !draftTitle.trim()" @click="savePrompt">保存</button>
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
        <section v-if="space === 'local' && collections.length" data-testid="collection-list">
          <h2>合集</h2>
          <details v-for="collection in collections" :key="collection.id">
            <summary>{{ collection.title }} · {{ collection.member_count }} 个提示词</summary>
            <button v-for="member in prompts.filter((row) => row.collection_id === collection.id)" :key="member.id" type="button" class="prompt-row" data-testid="collection-member" @click="openPrompt(member.id)">{{ member.title }}</button>
          </details>
        </section>
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
            <button type="button" class="primary-button" data-testid="wizard-copy" :disabled="copyBusy" @click="copyPreview">复制</button>
            <p v-if="copyNote" role="status" data-testid="copy-note">{{ copyNote }}</p>
          </div>
        </section>
        <p v-if="space === 'local' && !prompts.length" class="empty">浏览器内存库是空的。点「新建」只会写在这个标签页里。</p>
        <div v-else-if="space === 'square'" class="square-pane">
          <p v-if="squareOffline" data-testid="square-offline" class="empty">
            当前离线。预发广场暂时不可用，本地内存库仍可使用。
          </p>
          <button v-if="squareOffline" type="button" class="primary-button" data-testid="go-local" @click="openLocal">前往本地</button>
          <p v-if="favoriteNote" data-testid="favorite-note">{{ favoriteNote }}</p>
          <ul v-if="!squareOffline && squareItems.length" data-testid="square-list" class="prompt-list">
            <li v-for="item in squareItems" :key="item.id">
              <span>{{ item.title }}</span>
              <button type="button" data-testid="square-download" :disabled="downloadBusy.includes(item.id)" @click="downloadItem(item.id)">下载</button>
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
import { computed, onMounted, onUnmounted, ref } from "vue";
import { activateMemoryLibrary, markPromptSynced, createLocalPrompt, getLocalPrompt, listLocalCollections, listLocalPrompts, updateLocalPrompt } from "./memoryLibrary.js";
import { extractVariables, renderPrompt } from "./renderPrompt.js";
import { downloadSquareItem, listSquareItems, putFavorite } from "./square.js";
import { loadAccountLibrary, pushAccountPrompt } from "./accountLibrary.js";
import {
  getBillingStatus,
  redeemBillingCode,
  startBillingCheckout,
} from "./billing.js";
import {
  getSession,
  listOAuthProviders,
  loginOAuthSession,
  loginSession,
  logoutSession,
} from "./session.js";

const sidebarCollapsed = ref(false);
const space = ref("local");
const prompts = ref([]);
const collections = ref([]);
const downloadBusy = ref([]);
const cloudNote = ref("");
const syncBusy = ref(false);
const copyNote = ref("");
const copyBusy = ref(false);
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
const session = ref(getSession());
const billingPro = ref(false);
const billingMock = ref(false);
const billingMockPro = ref(false);
const billingBusy = ref(false);
const billingNote = ref("");
const redeemCode = ref("");
let loginAbort = new AbortController();

function reload() {
  prompts.value = listLocalPrompts();
  collections.value = listLocalCollections();
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
  copyNote.value = "";
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
  if (copyBusy.value) return;
  copyBusy.value = true;
  copyNote.value = "";
  try { await navigator.clipboard.writeText(previewText.value); copyNote.value = "已复制"; }
  catch { copyNote.value = "复制失败，请检查剪贴板权限后重试，填写内容已保留。"; }
  finally { copyBusy.value = false; }
}

function savePrompt() {
  if (!draftTitle.value.trim() || syncBusy.value) return;
  let saved;
  if (editingId.value) {
    saved = updateLocalPrompt({
      id: editingId.value,
      title: draftTitle.value,
      content: draftContent.value,
    });
  } else {
    saved = createLocalPrompt({ title: draftTitle.value, content: draftContent.value });
  }
  if (getSession().loggedIn) {
    saved.sync_pending = true;
  }
  const keepId = editingId.value || saved?.id;
  editing.value = false;
  editingId.value = null;
  draftTitle.value = "";
  draftContent.value = "";
  reload();
  opened.value = keepId ? getLocalPrompt(keepId) : (prompts.value[0] ?? null);
  if (getSession().loggedIn) retrySaves();
  else cloudNote.value = "已保存在当前标签页，刷新或关闭后会丢失。";
}

async function retrySaves() {
  if (syncBusy.value || !getSession().loggedIn) return;
  const token = getSession().accessToken;
  syncBusy.value = true;
  cloudNote.value = "正在保存到账号库…";
  try {
    for (const row of listLocalPrompts().filter((row) => row.sync_pending)) {
      if (getSession().accessToken !== token) return;
      const timestamp = row.updated_at;
      await pushAccountPrompt(row);
      if (getSession().accessToken !== token) return;
      markPromptSynced(row.id, timestamp);
    }
    cloudNote.value = "已保存到账号库。";
  } catch (error) {
    if (getSession().accessToken === token) cloudNote.value = `尚未保存到账号库，修改保留在本标签页：${error.message || error}`;
  } finally { syncBusy.value = false; reload(); }
}

async function readAccount() {
  const token = getSession().accessToken;
  try { await loadAccountLibrary(); if (getSession().accessToken === token) cloudNote.value = "已读取账号库，待同步修改仍保留。"; }
  catch (error) { if (getSession().accessToken === token) cloudNote.value = error.message || String(error); }
  finally {
    reload();
    if (opened.value && !editing.value && !using.value) opened.value = getLocalPrompt(opened.value.id);
  }
}

async function openLogin() {
  loginOpen.value = true;
  loginError.value = "";
  await loadProviders();
}

function cancelLogin() {
  loginAbort.abort();
  loginOpen.value = false;
  loginBusy.value = false;
  pendingFavorite.value = "";
}

async function signOut() {
  cancelLogin();
  const revoke = logoutSession();
  session.value = getSession();
  activateMemoryLibrary(null);
  editing.value = false;
  opened.value = null;
  using.value = false;
  billingPro.value = false;
  billingMock.value = false;
  billingMockPro.value = false;
  billingNote.value = "";
  favoriteNote.value = "";
  cloudNote.value = "已退出，已返回访客标签页内存库。";
  reload();
  try { await revoke; }
  catch (error) { cloudNote.value = error.message || String(error); }
}

function openLocal() {
  space.value = "local";
}

async function openSquare() {
  space.value = "square";
  favoriteNote.value = "";
  cancelLogin();
  squareOffline.value = false;
  try {
    squareItems.value = await listSquareItems();
  } catch {
    squareOffline.value = true;
    squareItems.value = [];
  }
}

async function downloadItem(id) {
  if (downloadBusy.value.includes(id)) return;
  downloadBusy.value = [...downloadBusy.value, id];
  try {
    await downloadSquareItem(id);
    reload();
    space.value = "local";
  } catch (error) { favoriteNote.value = `下载失败：${error.message || error}`; }
  finally { downloadBusy.value = downloadBusy.value.filter((item) => item !== id); }
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
  session.value = getSession();
  editing.value = false;
  opened.value = null;
  using.value = false;
  loginPassword.value = "";
  await readAccount();
  await loadBilling();
  const id = pendingFavorite.value;
  pendingFavorite.value = "";
  if (id) await favoriteItem(id);
}

async function submitLogin() {
  if (loginBusy.value) return;
  loginAbort.abort();
  const controller = new AbortController();
  loginAbort = controller;
  loginBusy.value = true;
  loginError.value = "";
  try {
    await loginSession({ email: loginEmail.value, password: loginPassword.value, signal: controller.signal });
    await afterLogin();
  } catch (caught) {
    if (!controller.signal.aborted) loginError.value = caught instanceof Error ? caught.message : String(caught);
  } finally { if (loginAbort === controller) loginBusy.value = false; }
}

async function submitOAuth(provider) {
  if (loginBusy.value) return;
  loginError.value = "";
  loginBusy.value = true;
  loginAbort.abort();
  const controller = new AbortController();
  loginAbort = controller;
  try {
    await loginOAuthSession(provider, { signal: controller.signal });
    await afterLogin();
  } catch (caught) {
    if (controller.signal.aborted) return;
    loginError.value = caught instanceof Error ? caught.message : String(caught);
  } finally { if (loginAbort === controller) loginBusy.value = false; }
}

onMounted(async () => {
  session.value = getSession();
  if (session.value.loggedIn) {
    await readAccount();
    await loadBilling();
    return;
  }
  reload();
});

onUnmounted(() => loginAbort.abort());

function applyBilling(payload) {
  billingPro.value = Boolean(payload?.pro);
  billingMock.value = Boolean(payload?.mock);
  billingMockPro.value = Boolean(payload?.mock_pro);
  billingNote.value = payload?.note ?? "";
}

async function loadBilling() {
  const token = getSession().accessToken;
  if (!getSession().loggedIn) {
    billingPro.value = false;
    billingNote.value = "";
    return;
  }
  try {
    const payload = await getBillingStatus();
    if (getSession().accessToken === token) applyBilling(payload);
  } catch (error) {
    billingNote.value = error instanceof Error ? error.message : String(error);
  }
}

async function runCheckout(mockOutcome) {
  if (billingBusy.value) return;
  const token = getSession().accessToken;
  if (!getSession().loggedIn) return;
  try {
    billingBusy.value = true;
    const payload = await startBillingCheckout(typeof mockOutcome === "string" ? mockOutcome : undefined);
    if (getSession().accessToken !== token) return;
    applyBilling(payload);
    const url = payload?.checkout_url;
    if (!payload?.mock && typeof url === "string" && url.startsWith("https://checkout.stripe.com/")) {
      window.open(url, "_blank", "noopener");
    }
  } catch (error) {
    if (getSession().accessToken === token) billingNote.value = error instanceof Error ? error.message : String(error);
  } finally {
    billingBusy.value = false;
  }
}

async function runRedeem() {
  if (billingBusy.value || billingMock.value) return;
  if (!getSession().loggedIn) return;
  try {
    applyBilling(await redeemBillingCode(redeemCode.value));
  } catch (error) {
    billingNote.value = error instanceof Error ? error.message : String(error);
  }
}
</script>
