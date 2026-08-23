<template>
  <main class="admin">
    <header class="admin-header">
      <p class="kicker">ADMIN</p>
      <h1>提示方舟管理台</h1>
      <p class="hint">本机预发。Refresh 不写入浏览器存储。</p>
    </header>

    <form v-if="!loggedIn" class="login" @submit.prevent="submitLogin">
      <label>
        <span>邮箱</span>
        <input v-model="email" type="email" data-testid="admin-email" autocomplete="username">
      </label>
      <label>
        <span>密码</span>
        <input v-model="password" type="password" data-testid="admin-password" autocomplete="current-password">
      </label>
      <p v-if="error" data-testid="admin-error">{{ error }}</p>
      <button type="button" data-testid="admin-login" @click="submitLogin">登录</button>
    </form>

    <section v-else>
      <p data-testid="admin-account">{{ account }}</p>
      <nav class="admin-nav">
        <button type="button" data-testid="nav-review" @click="page = 'review'">审核</button>
        <button type="button" data-testid="nav-users" @click="openUsers">用户</button>
        <button type="button" data-testid="nav-settings" @click="openSettings">设置</button>
      </nav>
      <p v-if="error" data-testid="admin-error">{{ error }}</p>
      <ul v-if="page === 'review'" data-testid="review-list" class="review-list">
        <li v-for="item in items" :key="item.id" class="review-row">
          <span>{{ item.source_id }}</span>
          <span>{{ item.status }}</span>
          <button type="button" data-testid="review-approve" @click="approve(item.id)">通过</button>
          <button type="button" data-testid="review-reject" @click="reject(item.id)">驳回</button>
        </li>
        <li v-if="items.length === 0">没有待审发布</li>
      </ul>
      <ul v-else-if="page === 'users'" data-testid="user-list" class="review-list">
        <li v-for="user in users" :key="user.email" class="review-row user-row">
          <span>{{ user.email }}</span>
          <span>{{ user.role }}</span>
        </li>
      </ul>
      <section v-else data-testid="settings-panel" class="settings">
        <label>
          <input v-model="squarePublic" type="checkbox" data-testid="setting-square-public">
          允许匿名浏览广场
        </label>
        <button type="button" data-testid="settings-save" @click="saveSettings">保存</button>
      </section>
    </section>
  </main>
</template>

<script setup>
import { onMounted, ref } from "vue";
import {
  approvePublication,
  getAdminSettings,
  listAdminUsers,
  listPendingPublications,
  putAdminSettings,
  rejectPublication,
} from "./adminApi.js";
import { getAdminSession, loginAdmin } from "./session.js";

const email = ref("");
const password = ref("");
const error = ref("");
const loggedIn = ref(false);
const account = ref("");
const items = ref([]);
const users = ref([]);
const page = ref("review");
const squarePublic = ref(true);

onMounted(() => {
  const session = getAdminSession();
  loggedIn.value = session.loggedIn;
  account.value = session.email ?? "";
});

async function submitLogin() {
  error.value = "";
  try {
    const session = await loginAdmin({ email: email.value, password: password.value });
    loggedIn.value = true;
    account.value = session.email;
    await refreshList();
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function refreshList() {
  const payload = await listPendingPublications();
  items.value = payload.items ?? [];
}

async function openSettings() {
  error.value = "";
  page.value = "settings";
  try {
    const payload = await getAdminSettings();
    squarePublic.value = Boolean(payload.square_public);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function saveSettings() {
  error.value = "";
  try {
    const payload = await putAdminSettings(squarePublic.value);
    squarePublic.value = Boolean(payload.square_public);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function openUsers() {
  error.value = "";
  page.value = "users";
  try {
    const payload = await listAdminUsers();
    users.value = payload.items ?? [];
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function approve(id) {
  error.value = "";
  try {
    await approvePublication(id);
    items.value = items.value.filter((item) => item.id !== id);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}

async function reject(id) {
  error.value = "";
  try {
    await rejectPublication(id);
    items.value = items.value.filter((item) => item.id !== id);
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : String(caught);
  }
}
</script>

<style>
:root {
  color-scheme: light;
  font-family: "SF Pro Text", "PingFang SC", sans-serif;
  background: #f4f1ea;
  color: #1c1914;
}

body {
  margin: 0;
}

.admin {
  max-width: 42rem;
  margin: 0 auto;
  padding: 2rem 1.25rem;
}

.kicker {
  letter-spacing: 0.12em;
  font-size: 0.75rem;
  margin: 0;
}

.hint,
.admin-header p {
  color: #5c564c;
}

.login,
.review-row {
  display: grid;
  gap: 0.75rem;
}

.login label {
  display: grid;
  gap: 0.35rem;
}

input,
button {
  font: inherit;
}

input {
  padding: 0.5rem 0.6rem;
}

button {
  padding: 0.45rem 0.8rem;
}

.review-list {
  list-style: none;
  padding: 0;
  display: grid;
  gap: 0.75rem;
}

.review-row {
  grid-template-columns: 1fr auto auto auto;
  align-items: center;
  background: #fff;
  padding: 0.75rem 1rem;
}

.user-row {
  grid-template-columns: 1fr auto;
}

.admin-nav {
  display: flex;
  gap: 0.5rem;
  margin: 0 0 1rem;
}

.settings {
  display: grid;
  gap: 0.75rem;
  background: #fff;
  padding: 0.75rem 1rem;
}
</style>
