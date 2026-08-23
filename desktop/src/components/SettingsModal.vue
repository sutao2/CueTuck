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
            <p v-if="prefError" data-testid="pref-error">{{ prefError }}</p>
            <label class="setting-row">
              <span class="setting-copy"><strong>开机启动</strong><small>登录系统后自动打开应用。未验证的系统不会声称已生效。</small></span>
              <input
                type="checkbox"
                data-testid="launch-at-login"
                :checked="launchAtLogin"
                @change="togglePref(DESKTOP_PREF_KEYS.launchAtLogin, $event)"
              >
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>关闭后最小化到托盘</strong><small>已验证的 macOS 会隐藏主窗口而不是退出。</small></span>
              <input
                type="checkbox"
                data-testid="minimize-to-tray"
                :checked="minimizeToTray"
                @change="togglePref(DESKTOP_PREF_KEYS.minimizeToTray, $event)"
              >
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>使用后自动关闭快捷窗口</strong><small>完成粘贴后收起启动器。</small></span>
              <input
                type="checkbox"
                data-testid="close-launcher-after-use"
                :checked="closeLauncherAfterUse"
                @change="togglePref(DESKTOP_PREF_KEYS.closeLauncherAfterUse, $event)"
              >
            </label>
          </section>
          <section v-else-if="current === 'account'">
            <h3>账号与广场</h3>
            <p>当前账号接到已有邮箱密码登录，不提供未选定的第三方绑定。</p>
            <div class="setting-row">
              <span class="setting-copy"><strong>当前账号</strong><small>使用工作台已有登录，不新增 OAuth。</small></span>
              <span class="setting-control">尚未接到本页</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>作者主页</strong><small>资料编辑尚未提供，不会假装已保存。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>我的发布</strong><small>远端发布列表尚未提供。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>下载时保留作者信息</strong><small>本机开关，未接通前只展示该行。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
          </section>
          <section v-else-if="current === 'shortcuts'">
            <h3>快捷键</h3>
            <p>登记全局组合以唤起独立启动器。与系统冲突时会提示，不会静默失效。</p>
            <label class="field">
              <span>唤起快捷搜索</span>
              <input v-model="shortcut" placeholder="Control+Space">
            </label>
            <div class="modal-actions">
              <button type="button" class="button primary-button" @click="saveShortcut">保存快捷键</button>
            </div>
            <p v-if="shortcutError" data-testid="shortcut-error">{{ shortcutError }}</p>
            <label class="field">
              <span>新建提示词</span>
              <input v-model="newPromptShortcut" data-testid="new-prompt-shortcut" placeholder="Control+Alt+N">
            </label>
            <label class="field">
              <span>快速粘贴最近使用</span>
              <input v-model="pasteRecentShortcut" data-testid="paste-recent-shortcut" placeholder="Control+Shift+V">
            </label>
          </section>
          <section v-else-if="current === 'sync'" data-testid="settings-unavailable">
            <h3>同步</h3>
            <p>云同步尚未提供。当前不会请求网络，也不会假装已接通。</p>
            <div class="setting-row">
              <span class="setting-copy"><strong>自动同步收藏与发布草稿</strong><small>联网队列尚未提供。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>仅在 Wi-Fi 下同步图片</strong><small>没有云同步引擎，不会按网络类型上传。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>冲突处理</strong><small>本地与远端同时修改时的策略尚未提供。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>立即同步</strong><small>不会向服务器发请求，也不会写入假的已同步状态。</small></span>
              <button type="button" class="button ghost-button" data-testid="sync-now" @click="noteSync">立即同步</button>
            </div>
            <p v-if="syncNote" data-testid="sync-note">{{ syncNote }}</p>
          </section>
          <section v-else-if="current === 'models'">
            <h3>AI 与模型</h3>
            <p>这些是本机目录、标签与建议，不会把提示词正文发到模型供应商。</p>
            <div class="setting-row">
              <span class="setting-copy"><strong>默认目标模型</strong><small>用于筛选和兼容性提示。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>已启用模型库</strong><small>本机目录，不外传正文。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>显示模型标签</strong><small>只影响本机卡片展示。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>变量智能建议</strong><small>关闭时不提供建议；打开也不上传正文。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>自定义模型列表</strong><small>管理本地可选名称。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
          </section>
          <section v-else-if="current === 'data'">
            <h3>数据与备份</h3>
            <div class="setting-row">
              <span class="setting-copy"><strong>SQLite 数据库</strong><small>打开库文件所在目录。未接通前只展示该行。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>导出完整备份</strong><small>ZIP 含提示词、合集、分类、封面与设置。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>自动备份</strong><small>额外调度，不替换 JSON 或库文件备份。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
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
          <section v-else-if="current === 'network'">
            <h3>网络与代理</h3>
            <div class="setting-row">
              <span class="setting-copy"><strong>允许访问提示词广场</strong><small>关闭后工作台不请求广场；启动器仍只搜本地。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>代理</strong><small>跟随系统。未提供手动配置前不假装自建代理。</small></span>
              <span class="setting-control">跟随系统</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>同步状态</strong><small>没有云同步，不会显示假的已同步。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
          </section>
          <section v-else-if="current === 'appearance'">
            <h3>外观</h3>
            <label class="field">
              <span>主题</span>
              <select :value="theme" @change="$emit('theme', $event.target.value)">
                <option value="light">浅色</option>
                <option value="dark">深色</option>
                <option value="system">跟随系统</option>
              </select>
            </label>
            <div class="setting-row">
              <span class="setting-copy"><strong>界面语言</strong><small>中文 / English。未接通前只展示该行。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>提示词双语版本</strong><small>关闭不删除已有中英正文。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>内容密度</strong><small>控制卡片间距。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
          </section>
          <section v-else-if="current === 'privacy'">
            <h3>隐私与安全</h3>
            <div class="setting-row">
              <span class="setting-copy"><strong>本地提示词默认不上传</strong><small>未点发布不得把本地正文送出。</small></span>
              <span class="setting-control">始终生效</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>匿名下载统计</strong><small>未接通前不会静默上报。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>清除使用历史</strong><small>只删最近使用记录，不删提示词正文。</small></span>
              <span class="setting-control">尚未接通</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>系统钥匙串</strong><small>Refresh 只在系统密钥库，不进 Web Storage。</small></span>
              <span class="setting-control">本机钥匙串</span>
            </div>
          </section>
          <section v-else-if="current === 'updates'" data-testid="settings-updates">
            <h3>更新</h3>
            <div class="setting-row">
              <span class="setting-copy"><strong>当前版本</strong><small>桌面包 {{ appVersion }}，与本机构建一致。</small></span>
              <button type="button" class="button ghost-button" data-testid="check-updates" @click="noteUpdates">检查更新</button>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>自动下载更新</strong><small>没有更新通道，不会后台安装。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>更新通道</strong><small>稳定版 / 预览版尚未提供。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>发行说明</strong><small>没有商店或更新服务器可查。</small></span>
              <span class="setting-control">尚未提供</span>
            </div>
            <p v-if="updateNote">{{ updateNote }}</p>
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
  setLocalSetting,
} from "../platform/library.js";
import { DEFAULT_LAUNCHER_SHORTCUT, DEFAULT_NEW_PROMPT_SHORTCUT, DEFAULT_PASTE_RECENT_SHORTCUT, registerLauncherShortcut } from "../platform/shortcut.js";
import pkg from "../../package.json";
import { DESKTOP_PREF_KEYS, isPrefOn, saveDesktopPref } from "../platform/desktopPrefs.js";

const props = defineProps({
  theme: { type: String, default: "light" },
  host: { type: String, default: "macos" },
});
const emit = defineEmits(["cancel", "theme", "imported"]);

const pages = [
  { id: "general", label: "常规" },
  { id: "account", label: "账号与广场" },
  { id: "shortcuts", label: "快捷键" },
  { id: "sync", label: "同步" },
  { id: "models", label: "AI 与模型" },
  { id: "data", label: "数据与备份" },
  { id: "network", label: "网络与代理" },
  { id: "appearance", label: "外观" },
  { id: "privacy", label: "隐私与安全" },
  { id: "updates", label: "更新" },
];
const current = ref("general");
const exportText = ref("");
const importText = ref("");
const preview = ref(null);
const shortcut = ref(DEFAULT_LAUNCHER_SHORTCUT);
const newPromptShortcut = ref(DEFAULT_NEW_PROMPT_SHORTCUT);
const pasteRecentShortcut = ref(DEFAULT_PASTE_RECENT_SHORTCUT);
const shortcutError = ref("");
const restorePath = ref("");
const backupPath = ref("");
const dataError = ref("");
const syncNote = ref("");
const updateNote = ref("");
const appVersion = pkg.version;
const prefError = ref("");
const launchAtLogin = ref(false);
const minimizeToTray = ref(false);
const closeLauncherAfterUse = ref(true);

onMounted(async () => {
  const stored = await getLocalSetting("launcher_shortcut");
  if (stored) shortcut.value = stored;
  const storedNew = await getLocalSetting("new_prompt_shortcut");
  if (storedNew) newPromptShortcut.value = storedNew;
  const storedPaste = await getLocalSetting("paste_recent_shortcut");
  if (storedPaste) pasteRecentShortcut.value = storedPaste;
  launchAtLogin.value = isPrefOn(await getLocalSetting(DESKTOP_PREF_KEYS.launchAtLogin));
  minimizeToTray.value = isPrefOn(await getLocalSetting(DESKTOP_PREF_KEYS.minimizeToTray));
  closeLauncherAfterUse.value = isPrefOn(
    await getLocalSetting(DESKTOP_PREF_KEYS.closeLauncherAfterUse),
    true,
  );
});

async function togglePref(key, event) {
  const enabled = event.target.checked;
  prefError.value = "";
  try {
    await saveDesktopPref(key, enabled, props.host, setLocalSetting);
    if (key === DESKTOP_PREF_KEYS.launchAtLogin) launchAtLogin.value = enabled;
    if (key === DESKTOP_PREF_KEYS.minimizeToTray) minimizeToTray.value = enabled;
    if (key === DESKTOP_PREF_KEYS.closeLauncherAfterUse) closeLauncherAfterUse.value = enabled;
  } catch (error) {
    prefError.value = error instanceof Error ? error.message : String(error);
    event.target.checked = false;
    if (key === DESKTOP_PREF_KEYS.launchAtLogin) launchAtLogin.value = false;
    if (key === DESKTOP_PREF_KEYS.minimizeToTray) minimizeToTray.value = false;
  }
}

function noteSync() {
  syncNote.value = "云同步尚未提供，没有向服务器发请求。";
}

function noteUpdates() {
  updateNote.value = "尚未配置更新通道，不会连接更新服务器或应用商店。";
}

async function saveShortcut() {
  shortcutError.value = "";
  try {
    const invokeCombo = shortcut.value.trim() || DEFAULT_LAUNCHER_SHORTCUT;
    const createCombo = newPromptShortcut.value.trim() || DEFAULT_NEW_PROMPT_SHORTCUT;
    const pasteCombo = pasteRecentShortcut.value.trim() || DEFAULT_PASTE_RECENT_SHORTCUT;
    await registerLauncherShortcut(invokeCombo, {
      extras: [
        {
          combo: createCombo,
          handler: async (event) => {
            if (event?.state && event.state !== "Pressed") return;
            const { invoke } = await import("@tauri-apps/api/core");
            await invoke("open_new_prompt");
          },
        },
        {
          combo: pasteCombo,
          handler: async (event) => {
            if (event?.state && event.state !== "Pressed") return;
            const { invoke } = await import("@tauri-apps/api/core");
            await invoke("paste_recent_prompt");
          },
        },
      ],
    });
    await setLocalSetting("new_prompt_shortcut", createCombo);
    await setLocalSetting("paste_recent_shortcut", pasteCombo);
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
