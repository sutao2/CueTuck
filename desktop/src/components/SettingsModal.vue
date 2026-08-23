<template>
  <div class="modal-layer" data-testid="settings-modal">
    <div class="modal-backdrop" @click="$emit('cancel')"></div>
    <section class="modal settings-modal" role="dialog" aria-modal="true">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">LOCAL SETTINGS</p>
          <h2>设置</h2>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" @click="$emit('cancel')">×</button>
      </header>
      <div class="settings-body">
        <nav class="settings-nav">
          <button
            v-for="page in pages"
            :key="page.id"
            type="button"
            :class="{ active: current === page.id }"
            :data-settings-page="page.id"
            @click="current = page.id"
          >
            {{ page.label }}
          </button>
        </nav>
        <div class="settings-content">
          <section v-if="current === 'general'">
            <h3>常规</h3>
            <p>第一期只使用本机库。启动器仍是独立窗口。</p>
          </section>
          <section v-else-if="current === 'shortcuts'">
            <h3>快捷键</h3>
            <p>登记全局组合以唤起独立启动器。与系统冲突时会提示，不会静默失效。</p>
            <label class="field">
              <span>启动器快捷键</span>
              <input v-model="shortcut" placeholder="Control+Space">
            </label>
            <div class="modal-actions">
              <button type="button" class="button primary-button" @click="saveShortcut">保存快捷键</button>
            </div>
            <p v-if="shortcutError" data-testid="shortcut-error">{{ shortcutError }}</p>
          </section>
          <section v-else-if="current === 'data'">
            <h3>数据与备份</h3>
            <div class="modal-actions">
              <button type="button" class="button ghost-button" @click="doExport">导出 JSON</button>
            </div>
            <textarea v-model="exportText" rows="6" readonly></textarea>
            <label class="field">
              <span>导入 JSON</span>
              <textarea v-model="importText" rows="5" placeholder='{"prompts":[{"title":"一","content":"a"}]}'></textarea>
            </label>
            <div class="modal-actions">
              <button type="button" class="button ghost-button" @click="doPreview">预览</button>
              <button type="button" class="button primary-button" :disabled="!preview" @click="doApply">确认导入</button>
            </div>
            <p v-if="preview" data-testid="import-preview">
              将导入 {{ preview.prompt_count }} 条提示词、{{ preview.collection_count }} 个合集。确认前不会写入。
            </p>
            <label class="field">
              <span>恢复库文件路径</span>
              <input v-model="restorePath" placeholder="/path/to/promptark.sqlite">
            </label>
            <div class="modal-actions">
              <button type="button" class="button ghost-button" @click="doBackup">备份库文件</button>
              <button type="button" class="button primary-button" @click="doRestore">恢复库文件</button>
            </div>
            <p v-if="backupPath" data-testid="backup-path">已备份到 {{ backupPath }}</p>
            <p v-if="dataError" data-testid="backup-error">{{ dataError }}</p>
          </section>
          <section v-else-if="current === 'appearance'">
            <h3>外观</h3>
            <label class="field">
              <span>主题</span>
              <select :value="theme" @change="$emit('theme', $event.target.value)">
                <option value="light">浅色</option>
                <option value="dark">深色</option>
              </select>
            </label>
          </section>
          <section v-else data-testid="settings-unavailable">
            <h3>{{ pages.find((page) => page.id === current)?.label }}</h3>
            <p>将在联网版提供。当前不会请求网络，也不会假装已接通。</p>
          </section>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup>
import { onMounted, ref } from "vue";
import {
  applyLocalImport,
  backupLocalLibrary,
  exportLocalLibrary,
  getLocalSetting,
  previewLocalImport,
  restoreLocalLibrary,
} from "../platform/library.js";
import { DEFAULT_LAUNCHER_SHORTCUT, registerLauncherShortcut } from "../platform/shortcut.js";

defineProps({
  theme: { type: String, default: "light" },
});
const emit = defineEmits(["cancel", "theme", "imported"]);

const pages = [
  { id: "general", label: "常规" },
  { id: "shortcuts", label: "快捷键" },
  { id: "data", label: "数据与备份" },
  { id: "appearance", label: "外观" },
  { id: "sync", label: "同步" },
  { id: "account", label: "账号与广场" },
];
const current = ref("general");
const exportText = ref("");
const importText = ref("");
const preview = ref(null);
const shortcut = ref(DEFAULT_LAUNCHER_SHORTCUT);
const shortcutError = ref("");
const restorePath = ref("");
const backupPath = ref("");
const dataError = ref("");

onMounted(async () => {
  const stored = await getLocalSetting("launcher_shortcut");
  if (stored) shortcut.value = stored;
});

async function saveShortcut() {
  shortcutError.value = "";
  try {
    await registerLauncherShortcut(shortcut.value.trim() || DEFAULT_LAUNCHER_SHORTCUT);
  } catch (error) {
    shortcutError.value = error instanceof Error ? error.message : String(error);
  }
}

async function doExport() {
  exportText.value = await exportLocalLibrary();
}

async function doPreview() {
  preview.value = await previewLocalImport(importText.value);
}

async function doApply() {
  await applyLocalImport(importText.value);
  emit("imported");
}

async function doBackup() {
  dataError.value = "";
  backupPath.value = "";
  try {
    backupPath.value = await backupLocalLibrary();
  } catch (error) {
    dataError.value = error instanceof Error ? error.message : String(error);
  }
}

async function doRestore() {
  dataError.value = "";
  try {
    await restoreLocalLibrary(restorePath.value.trim());
    emit("imported");
  } catch (error) {
    dataError.value = error instanceof Error ? error.message : String(error);
  }
}
</script>
