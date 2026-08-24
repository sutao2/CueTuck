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
        <button type="button" class="space-tab" data-space="square" :class="{ active: space === 'square' }" @click="space = 'square'">
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
          <li v-for="row in prompts" :key="row.id">{{ row.title }}</li>
        </ul>
        <p v-else-if="space === 'local'" class="empty">浏览器内存库是空的。点「新建」只会写在这个标签页里。</p>
        <p v-else class="empty">广场仍走本仓库预发 API。未开后端时列表为空。</p>
      </main>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref } from "vue";
import { createLocalPrompt, listLocalPrompts } from "./memoryLibrary.js";

const sidebarCollapsed = ref(false);
const space = ref("local");
const prompts = ref([]);
const editing = ref(false);
const draftTitle = ref("");
const draftContent = ref("");

function reload() {
  prompts.value = listLocalPrompts();
}

function startCreate() {
  editing.value = true;
  draftTitle.value = "";
  draftContent.value = "";
}

function savePrompt() {
  if (!draftTitle.value.trim()) return;
  createLocalPrompt({ title: draftTitle.value, content: draftContent.value });
  editing.value = false;
  draftTitle.value = "";
  draftContent.value = "";
  reload();
}

onMounted(reload);
</script>
