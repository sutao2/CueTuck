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
        <h1>{{ space === "local" ? "本地提示词" : "提示词广场" }}</h1>
        <p v-if="space === 'local'" class="empty">浏览器内存库是空的。点「新建」只会写在这个标签页里。</p>
        <p v-else class="empty">广场仍走本仓库预发 API。未开后端时列表为空。</p>
      </main>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";

const sidebarCollapsed = ref(false);
const space = ref("local");
</script>
