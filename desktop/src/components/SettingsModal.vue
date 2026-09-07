<template>
  <div class="modal-layer" data-testid="settings-modal" @keydown.esc="onEscape">
    <div class="modal-backdrop" @click="requestClose"></div>
    <section v-dialog-focus="requestClose" class="modal settings-modal" role="dialog" aria-modal="true" aria-labelledby="settings-title">
      <button type="button" class="modal-close settings-close" aria-label="关闭" @click="requestClose">×</button>
      <div class="settings-body">
        <nav class="settings-nav" aria-labelledby="settings-title">
          <h2 id="settings-title">{{ uiText(uiLanguage, "settings") }}</h2>
          <button
            v-for="page in pages"
            :key="page.id"
            type="button"
            :class="{ active: current === page.id }"
            :data-settings-page="page.id"
            :aria-current="current === page.id ? 'page' : undefined"
            @click="current = page.id"
          >
            <AppIcon :name="page.icon" /><span>{{ page.label }}</span>
          </button>
        </nav>
        <div :key="current" class="settings-content">
          <fieldset class="settings-fields" :disabled="saving || loading">
          <section v-if="current === 'general'">
            <h3>常规</h3>
            <p>管理应用启动、托盘和快捷窗口的使用偏好。</p>
            <p v-if="prefError" data-testid="pref-error">{{ prefError }}</p>
            <label class="setting-row">
              <span class="setting-copy"><strong>开机启动</strong><small>登录系统后自动打开提示方舟。</small></span>
              <input
                type="checkbox"
                data-testid="launch-at-login"
                :checked="launchAtLogin"
                @change="togglePref(DESKTOP_PREF_KEYS.launchAtLogin, $event)"
              >
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>关闭后最小化到托盘</strong><small>关闭主窗口后保留在托盘，方便随时返回。</small></span>
              <input
                type="checkbox"
                data-testid="minimize-to-tray"
                :checked="minimizeToTray"
                @change="togglePref(DESKTOP_PREF_KEYS.minimizeToTray, $event)"
              >
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>使用后自动关闭快捷窗口</strong><small>复制或粘贴成功后收起启动器，失败时保留填写内容。</small></span>
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
            <p>管理登录账号、公开资料与订阅。</p>
            <div class="setting-row">
              <span class="setting-copy"><strong>当前账号</strong><small>登录后可发布、收藏和同步提示词。</small></span>
              <span class="setting-control account-session">
                <span data-testid="current-account">{{ session.loggedIn ? session.email : "未登录" }}</span>
                <button v-if="!session.loggedIn" type="button" class="button primary-button" data-testid="settings-login" @click="$emit('login')">登录</button>
                <button v-else type="button" class="button ghost-button" data-testid="settings-logout" @click="$emit('logout')">退出</button>
              </span>
            </div>
            <div class="setting-row setting-block">
              <span class="setting-copy"><strong>作者主页</strong><small>已登录可保存显示名与简介。</small></span>
              <div class="setting-control author-profile">
                <label class="profile-field"><span>显示名</span>
                <input
                  data-testid="author-display-name"
                  :disabled="!session.loggedIn"
                  v-model="displayName"
                  placeholder="显示名"
                >
                </label>
                <label class="profile-field"><span>简介</span>
                <textarea
                  data-testid="author-bio"
                  :disabled="!session.loggedIn"
                  v-model="bio"
                  placeholder="简介"
                ></textarea>
                </label>
                <button
                  type="button"
                  class="button primary-button"
                  data-testid="save-author-profile"
                  :disabled="!session.loggedIn"
                  @click="saveAuthorProfile"
                >
                  保存
                </button>
                <small v-if="profileNote" data-testid="author-profile-note">{{ profileNote }}</small>
              </div>
            </div>
            <div class="setting-row publications-row">
              <span class="setting-copy"><strong>我的发布</strong><small>当前账号提交到广场的审核状态。</small></span>
              <span class="setting-control" data-testid="my-publications">
                <template v-if="!session.loggedIn">未登录</template>
                <template v-else-if="!myPublications.length">暂无投稿</template>
                <ul v-else class="mine-list">
                  <li v-for="row in myPublications" :key="row.id">
                    {{ row.title || row.source_id }} · {{ row.status }}
                  </li>
                </ul>
              </span>
            </div>
            <div class="setting-row setting-block">
              <span class="setting-copy"><strong>账单</strong><small>预发可查状态、兑换码；只有测试密钥才跳转 Checkout。不是公开售卖。</small></span>
              <div class="setting-control author-profile">
                <span data-testid="billing-pro">{{ session.loggedIn ? (billingPro ? "Pro" : "未订阅") : "未登录" }}</span>
                <small v-if="session.loggedIn && billingMock" data-testid="billing-mock">Mock · {{ billingMockPro ? "模拟 Pro" : "模拟未订阅" }}（不扣款，不改变真实权益）</small>
            <small v-if="billingNote" data-testid="billing-note">{{ billingNote }}</small>
            <span v-if="session.loggedIn && billingMock">
              <button v-for="(label, outcome) in { success: '模拟成功', failure: '模拟失败', cancel: '模拟取消', reset: '重置模拟' }" :key="outcome" type="button" :data-testid="`billing-mock-${outcome}`" :disabled="billingBusy" @click="runCheckout(outcome)">{{ label }}</button>
            </span>
                <label class="profile-field"><span>兑换码</span>
                <input
                  data-testid="billing-redeem-code"
                  :disabled="!session.loggedIn || billingMock || billingBusy"
                  v-model="redeemCode"
                  placeholder="兑换码"
                >
                </label>
                <button
                  type="button"
                  class="button ghost-button"
                  data-testid="billing-redeem"
                  :disabled="!session.loggedIn || billingMock || billingBusy"
                  @click="runRedeem"
                >
                  兑换
                </button>
                <button
                  type="button"
                  class="button primary-button"
                  data-testid="billing-checkout"
                  v-if="!billingMock"
                  :disabled="!session.loggedIn || billingBusy"
                  @click="runCheckout"
                >
                  前往支付
                </button>
              </div>
            </div>
            <label class="setting-row">
              <span class="setting-copy"><strong>下载时保留作者信息</strong><small>打开后，新下载的本地副本展示作者，不改正文。</small></span>
              <input
                type="checkbox"
                data-testid="keep-author-on-download"
                :checked="keepAuthorOnDownload"
                @change="toggleKeepAuthorOnDownload"
              >
            </label>
          </section>
          <section v-else-if="current === 'shortcuts'">
            <h3>快捷键</h3>
            <p>登记全局组合以唤起独立启动器。与系统冲突时会提示，不会静默失效。</p>
            <p class="save-mode-hint">点击后按组合键，完成后保存。支持 Ctrl / Alt / Command 组合或 F1–F24；Tab 切换，Esc 取消。系统保留组合可能被拦截。</p>
            <label class="field">
              <span>唤起启动器</span>
              <ShortcutInput v-model="shortcut" :host="host" data-testid="launcher-shortcut" />
            </label>
            <label class="field">
              <span>新建提示词</span>
              <ShortcutInput v-model="newPromptShortcut" :host="host" data-testid="new-prompt-shortcut" />
            </label>
            <label class="field">
              <span>快速粘贴最近使用</span>
              <ShortcutInput v-model="pasteRecentShortcut" :host="host" data-testid="paste-recent-shortcut" />
            </label>
            <div class="modal-actions">
              <button type="button" class="button primary-button" @click="saveShortcut">保存快捷键</button>
            </div>
            <p v-if="shortcutError" data-testid="shortcut-error">{{ shortcutError }}</p>
          </section>
          <section v-else-if="current === 'sync'" data-testid="settings-unavailable">
            <h3>同步</h3>
            <p>已登录可立即同步个人库。启动器与 MCP 仍只读本机 SQLite。</p>
            <label class="setting-row" data-testid="auto-sync-queue-row">
              <span class="setting-copy"><strong>自动同步收藏与发布草稿</strong><small>打开后，收藏或发布在断网时写入本机队列，联网后随立即同步送出。不会假装已经到达服务器。</small></span>
              <input
                type="checkbox"
                data-testid="auto-sync-queue"
                :checked="autoSyncQueue"
                @change="toggleAutoSyncQueue"
              >
            </label>
            <label class="setting-row" data-testid="sync-wifi-images-row">
              <span class="setting-copy"><strong>仅在 Wi-Fi 下同步图片</strong><small>打开后，立即同步只在判定为 Wi-Fi 时推送封面。无法判定或非 Wi-Fi 时跳过封面，仍同步标题与正文。本机封面仍在。</small></span>
              <input
                type="checkbox"
                data-testid="sync-wifi-images"
                :checked="syncWifiImages"
                @change="toggleSyncWifiImages"
              >
            </label>
            <div class="setting-row" data-testid="sync-conflict">
              <span class="setting-copy"><strong>冲突处理</strong><small>立即同步默认按较新的 updated_at 覆盖。选择保留本地时不覆盖已有本机正文，远端独有条目仍写入。</small></span>
              <select data-testid="sync-conflict-strategy" :value="syncConflict" @change="saveSyncConflict">
                <option value="newer">较新者胜</option>
                <option value="keep_local">保留本地</option>
              </select>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>立即同步</strong><small>已登录时推拉账号库。未登录打开登录，不会假装已同步。</small></span>
              <button type="button" class="button ghost-button" data-testid="sync-now" @click="runSyncNow">立即同步</button>
            </div>
            <p v-if="syncNote" data-testid="sync-note">{{ syncNote }}</p>
          </section>
          <section v-else-if="current === 'models'">
            <h3>AI 与模型</h3>
            <p>这些是本机目录、标签与建议，不会把提示词正文发到模型供应商。</p>
            <p class="save-mode-hint">本页修改后请点击「保存本机模型偏好」。</p>
            <label class="field">
              <span>默认目标模型</span>
              <input v-model="defaultModel" data-testid="default-model" placeholder="本机目录名称">
            </label>
            <label class="field">
              <span>已启用模型库</span>
              <textarea v-model="modelCatalog" data-testid="model-catalog" rows="3" placeholder="每行一个本机模型名"></textarea>
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>显示模型标签</strong><small>只影响本机卡片展示。</small></span>
              <input type="checkbox" data-testid="show-model-tags" v-model="showModelTags">
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>变量智能建议</strong><small>关闭时不提供建议；打开也不上传正文。</small></span>
              <input type="checkbox" data-testid="variable-hints" v-model="variableHints">
            </label>
            <label class="field">
              <span>自定义模型列表</span>
              <textarea v-model="customModels" data-testid="custom-models" rows="2" placeholder="本机自定义名称"></textarea>
            </label>
            <div class="modal-actions">
              <button type="button" class="button primary-button" data-testid="save-models" @click="saveModels">保存本机模型偏好</button>
            </div>
          </section>
          <section v-else-if="current === 'data'">
            <h3>数据与备份</h3>
            <p>管理本机资料、导入导出与数据恢复。</p>
            <div class="setting-row">
              <span class="setting-copy"><strong>SQLite 数据库</strong><small>打开库文件所在目录。</small></span>
              <button type="button" class="button ghost-button" data-testid="open-library-dir" @click="openDir">打开目录</button>
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>导出完整备份</strong><small>ZIP 含库文件与 JSON，保留封面引用；不包含外部图片文件。</small></span>
              <button type="button" class="button ghost-button" data-testid="export-zip" @click="doZip">导出 ZIP</button>
            </div>
            <label class="setting-row">
              <span class="setting-copy"><strong>自动备份</strong><small>仅桌面：开启即备份，应用运行时每 24 小时备份一次，保留历史文件。</small></span>
              <input type="checkbox" data-testid="auto-backup" :checked="autoBackup" :disabled="dataBusy" @change="toggleAutoBackup">
            </label>
            <p v-if="autoBackupNote" role="status" data-testid="auto-backup-note">{{ autoBackupNote }}</p>
            <p v-if="zipPath" data-testid="zip-path">{{ zipPath }}</p>
            <div class="setting-row">
              <span class="setting-copy"><strong>导出 JSON</strong><small>导出可阅读、可再次导入的提示词数据。</small></span>
              <button type="button" class="button ghost-button" @click="doExport">导出 JSON</button>
            </div>
            <textarea v-if="exportText" v-model="exportText" rows="6" aria-label="导出的 JSON" readonly></textarea>
            <label class="field">
              <span>导入 JSON</span>
              <textarea v-model="importText" :disabled="importBusy" rows="5" placeholder='{"prompts":[{"title":"一","content":"a"}]}' @input="preview = null"></textarea>
            </label>
            <div class="modal-actions">
              <button type="button" class="button ghost-button" :disabled="importBusy" @click="doPreview">预览</button>
              <button type="button" class="button primary-button" :disabled="importBusy || !preview" @click="doApply">确认导入</button>
            </div>
            <p v-if="preview" data-testid="import-preview">
              将导入 {{ preview.prompt_count }} 条提示词、{{ preview.collection_count }} 个合集。确认前不会写入。
            </p>
            <p v-if="importNote" role="status">{{ importNote }}</p>
            <label class="field">
              <span>恢复库文件路径</span>
              <input v-model="restorePath" placeholder="/path/to/promptark.sqlite">
            </label>
            <div class="modal-actions">
              <button type="button" class="button ghost-button" :disabled="dataBusy" @click="doBackup">备份库文件</button>
              <button type="button" class="button danger-button" :disabled="dataBusy || !restorePath.trim()" @click="pendingAction = 'restore'">恢复库文件</button>
            </div>
            <p v-if="backupPath" data-testid="backup-path">已备份到 {{ backupPath }}</p>
            <p v-if="dataError" data-testid="backup-error">{{ dataError }}</p>
          </section>
          <section v-else-if="current === 'network'">
            <h3>网络与代理</h3>
            <p>控制联网范围，以及桌面端使用的代理。</p>
            <label class="setting-row">
              <span class="setting-copy"><strong>允许访问提示词广场</strong><small>关闭后工作台不请求广场；启动器仍只搜本地。</small></span>
              <input type="checkbox" data-testid="square-access" :checked="squareAccess" @change="toggleSquareAccess">
            </label>
            <label class="setting-row setting-block" data-testid="proxy-row">
              <span class="setting-copy"><strong>代理</strong><small>空则跟随系统。填写 http 或 https 地址后，本机请求走该代理。浏览器预览不走该代理。</small></span>
              <input
                data-testid="http-proxy"
                v-model="httpProxy"
                placeholder="跟随系统"
                @change="saveHttpProxy"
              >
            </label>
            <p v-if="proxyError" data-testid="proxy-error">{{ proxyError }}</p>
            <div class="setting-row" data-testid="sync-status">
              <span class="setting-copy"><strong>同步状态</strong><small>个人库可立即同步。没有后台自动同步，不会显示假进度。</small></span>
              <span class="setting-control">手动立即同步</span>
            </div>
          </section>
          <section v-else-if="current === 'appearance'">
            <h3>外观</h3>
            <p>选择适合你的主题、语言与内容密度。</p>
            <label class="field">
              <span>主题</span>
              <select data-testid="theme-select" :value="themeChoice" @change="saveTheme($event)">
                <option value="light">浅色</option>
                <option value="dark">深色</option>
                <option value="system">跟随系统</option>
              </select>
            </label>
            <label class="field">
              <span>界面语言</span>
              <select data-testid="ui-language" :value="uiLanguage" @change="saveUiLanguage($event.target.value, $event)">
                <option value="zh">中文</option>
                <option value="en">English</option>
              </select>
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>提示词双语版本</strong><small>关闭不删除已有中英正文。</small></span>
              <input type="checkbox" data-testid="prompt-bilingual" :checked="promptBilingual" @change="toggleBilingual">
            </label>
            <label class="field">
              <span>内容密度</span>
              <select data-testid="density" :value="density" @change="saveDensity($event.target.value, $event)">
                <option value="comfortable">舒适</option>
                <option value="compact">紧凑</option>
              </select>
            </label>
          </section>
          <section v-else-if="current === 'privacy'">
            <h3>隐私与安全</h3>
            <p>了解数据的保存方式，管理统计与使用记录。</p>
            <div class="setting-row">
              <span class="setting-copy"><strong>本地提示词默认不上传</strong><small>未点发布不得把本地正文送出。</small></span>
              <span class="setting-control">始终生效</span>
            </div>
            <label class="setting-row" data-testid="anonymous-download-stats-row">
              <span class="setting-copy"><strong>匿名下载统计</strong><small>打开后，成功下载只上报条目 id，不含账号、正文或标题。关闭时不请求。统计失败不影响下载。</small></span>
              <input
                type="checkbox"
                data-testid="anonymous-download-stats"
                :checked="anonymousDownloadStats"
                @change="toggleAnonymousDownloadStats"
              >
            </label>
            <div class="setting-row">
              <span class="setting-copy"><strong>清除使用历史</strong><small>只删最近使用记录，不删提示词正文。</small></span>
              <button type="button" class="button danger-button" data-testid="clear-use-history" @click="pendingAction = 'history'">清除</button>
            </div>
            <div class="setting-row" data-testid="keychain-row">
              <span class="setting-copy">
                <strong>系统钥匙串</strong>
                <small v-if="usesSystemKeychain()">Refresh 只在系统密钥库，不进 Web Storage。</small>
                <small v-else>浏览器预览没有系统密钥库。Refresh 不进 Web Storage。</small>
              </span>
              <span class="setting-control">{{ usesSystemKeychain() ? "本机钥匙串" : "浏览器内存" }}</span>
            </div>
          </section>
          <section v-else-if="current === 'updates'" data-testid="settings-updates">
            <h3>更新</h3>
            <p>管理版本、更新通道与发行说明。</p>
            <div class="setting-row">
              <span class="setting-copy"><strong>当前版本</strong><small>桌面包 {{ appVersion }}，与本机构建一致。</small></span>
              <button type="button" class="button ghost-button" data-testid="check-updates" @click="runCheckUpdates">检查更新</button>
            </div>
            <label class="setting-row">
              <span class="setting-copy"><strong>自动下载更新</strong><small>打开后，检查到当前通道有包时通过 updater 排队安装，不走应用商店。</small></span>
              <input
                type="checkbox"
                data-testid="auto-download"
                :checked="autoDownload"
                @change="toggleAutoDownload"
              >
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>更新通道</strong><small>稳定版只用正式发行，预览版只用预发行。</small></span>
              <select data-testid="update-channel" :value="updateChannel" @change="saveUpdateChannel">
                <option value="stable">稳定版</option>
                <option value="preview">预览版</option>
              </select>
            </label>
            <div class="setting-row">
              <span class="setting-copy"><strong>发行说明</strong><small>随检查更新展示，不来自应用商店。</small></span>
              <span class="setting-control">GitHub Releases</span>
            </div>
            <p v-if="releaseNotes" data-testid="release-notes">{{ releaseNotes }}</p>
            <p v-if="updateNote" data-testid="update-note">{{ updateNote }}</p>
          </section>
          </fieldset>
        </div>
      </div>
      <footer class="settings-feedback" :class="{ error: feedbackError }" role="status" aria-live="polite" data-testid="settings-feedback">
        {{ loading ? '正在读取设置…' : saving ? '正在保存…' : feedback || '开关与选择项即时保存；有保存按钮的表单需手动保存。' }}
      </footer>
      <div v-if="pendingAction" class="settings-confirm-layer">
        <section class="settings-confirm" role="alertdialog" aria-modal="true" aria-labelledby="settings-confirm-title" aria-describedby="settings-confirm-copy" @keydown.tab="trapConfirmationFocus">
          <h3 id="settings-confirm-title">{{ pendingAction === 'discard' ? '放弃未保存的修改？' : pendingAction === 'restore' ? '恢复数据库？' : '清除使用历史？' }}</h3>
          <p id="settings-confirm-copy">{{ pendingAction === 'discard' ? '尚未保存的表单和导入文本将被丢弃，已经保存的设置不受影响。' : pendingAction === 'restore' ? `将使用 ${restorePath} 替换当前库。请先备份当前数据，此操作不是合并导入。` : '将清除最近使用记录与使用次数，不删除提示词正文。' }}</p>
          <div class="modal-actions">
            <button ref="confirmCancel" type="button" class="button" data-testid="cancel-settings-action" @click="pendingAction = null">{{ pendingAction === 'discard' ? '继续编辑' : '取消' }}</button>
            <button type="button" class="button danger-button" data-testid="confirm-settings-action" @click="confirmAction">{{ pendingAction === 'discard' ? '放弃修改' : '确认执行' }}</button>
          </div>
        </section>
      </div>
    </section>
  </div>
</template>

<script setup>
import AppIcon from "./AppIcon.vue";
import { vDialogFocus } from "../lib/dialogFocus.js";
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { uiText } from "../platform/uiStrings.js";
import {
  applyLocalImport,
  backupLocalLibrary,
  setAutoBackup,
  clearLocalPromptUse,
  exportLibraryZip,
  exportLocalLibrary,
  getLocalSetting,
  openLibraryDir,
  previewLocalImport,
  restoreLocalLibrary,
  setLocalSetting,
} from "../platform/library.js";
import { DEFAULT_LAUNCHER_SHORTCUT, DEFAULT_NEW_PROMPT_SHORTCUT, DEFAULT_PASTE_RECENT_SHORTCUT, registerLauncherShortcut } from "../platform/shortcut.js";
import ShortcutInput from "./ShortcutInput.vue";
import pkg from "../../package.json";
import { DESKTOP_PREF_KEYS, isPrefOn, saveDesktopPref } from "../platform/desktopPrefs.js";
import { listMyPublications } from "../platform/square.js";
import { getMe, putMe, getSession } from "../platform/session.js";
import {
  getBillingStatus,
  redeemBillingCode,
  startBillingCheckout,
} from "../platform/billing.js";
import { syncLocalLibraryNow } from "../platform/librarySync.js";
import { parseHttpProxy } from "../platform/httpProxy.js";
import { flushSyncQueue } from "../platform/syncQueue.js";
import { checkForUpdates, queueUpdateInstall } from "../platform/updates.js";

const props = defineProps({
  theme: { type: String, default: "light" },
  host: { type: String, default: "macos" },
  session: { type: Object, default: () => ({ loggedIn: false, email: "" }) },
  language: { type: String, default: "zh" },
});
const emit = defineEmits(["cancel", "theme", "imported", "login", "logout", "history-cleared", "language", "launcher-shortcut-saved"]);

function usesSystemKeychain() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

const uiLanguage = ref(props.language);
const themeChoice = ref(props.theme);
watch(() => props.theme, (value) => { themeChoice.value = value; });
const pages = computed(() => [
  { id: "general", icon: "settings", label: uiText(uiLanguage.value, "settingsGeneral") },
  { id: "account", icon: "user", label: uiText(uiLanguage.value, "settingsAccount") },
  { id: "shortcuts", icon: "keyboard", label: uiText(uiLanguage.value, "settingsShortcuts") },
  { id: "sync", icon: "refresh", label: uiText(uiLanguage.value, "settingsSync") },
  { id: "models", icon: "models", label: uiText(uiLanguage.value, "settingsModels") },
  { id: "data", icon: "database", label: uiText(uiLanguage.value, "settingsData") },
  { id: "network", icon: "globe", label: uiText(uiLanguage.value, "settingsNetwork") },
  { id: "appearance", icon: "sun", label: uiText(uiLanguage.value, "settingsAppearance") },
  { id: "privacy", icon: "shield", label: uiText(uiLanguage.value, "settingsPrivacy") },
  { id: "updates", icon: "download", label: uiText(uiLanguage.value, "settingsUpdates") },
]);
const current = ref("general");
const exportText = ref("");
const importText = ref("");
const preview = ref(null);
const previewedText = ref("");
const importBusy = ref(false);
const importNote = ref("");
const shortcut = ref(DEFAULT_LAUNCHER_SHORTCUT);
const newPromptShortcut = ref(DEFAULT_NEW_PROMPT_SHORTCUT);
const pasteRecentShortcut = ref(DEFAULT_PASTE_RECENT_SHORTCUT);
const shortcutError = ref("");
const restorePath = ref("");
const backupPath = ref("");
const dataBusy = ref(false);
const autoBackupNote = ref("");
const dataError = ref("");
const syncNote = ref("");
const updateNote = ref("");
const releaseNotes = ref("");
const autoDownload = ref(false);
const updateChannel = ref("stable");
const syncConflict = ref("newer");
const syncWifiImages = ref(false);
const autoSyncQueue = ref(false);
const anonymousDownloadStats = ref(false);
const httpProxy = ref("");
const proxyError = ref("");
const appVersion = pkg.version;
const prefError = ref("");
const launchAtLogin = ref(false);
const minimizeToTray = ref(false);
const closeLauncherAfterUse = ref(true);
const autoBackup = ref(false);
const zipPath = ref("");
const squareAccess = ref(true);
const promptBilingual = ref(true);
const density = ref("comfortable");
const defaultModel = ref("");
const modelCatalog = ref("");
const showModelTags = ref(true);
const variableHints = ref(false);
const customModels = ref("");
const keepAuthorOnDownload = ref(false);
const myPublications = ref([]);
const billingPro = ref(false);
const billingMock = ref(false);
const billingMockPro = ref(false);
const billingBusy = ref(false);
const billingNote = ref("");
const redeemCode = ref("");
const displayName = ref("");
const bio = ref("");
const profileNote = ref("");
const saving = ref(false);
const loading = ref(true);
const feedback = ref("");
const feedbackError = ref(false);
const pendingAction = ref(null);
const confirmCancel = ref(null);
const savedDrafts = ref({});
let confirmationReturnFocus = null;
const modelDraft = () => JSON.stringify([defaultModel.value, modelCatalog.value, customModels.value, showModelTags.value, variableHints.value]);
const shortcutDraft = () => JSON.stringify([shortcut.value, newPromptShortcut.value, pasteRecentShortcut.value]);
const profileDraft = () => JSON.stringify([displayName.value, bio.value]);
const hasUnsaved = computed(() => !loading.value && (
  modelDraft() !== savedDrafts.value.models || shortcutDraft() !== savedDrafts.value.shortcuts ||
  profileDraft() !== savedDrafts.value.profile || Boolean(importText.value.trim())
));

function showFeedback(message, error = false) { feedback.value = message; feedbackError.value = error; }
function onEscape(event) {
  if (event.isComposing) return;
  event.preventDefault();
  event.stopPropagation();
  if (pendingAction.value) pendingAction.value = null;
  else requestClose();
}
function requestClose() {
  if (saving.value || loading.value || dataBusy.value || importBusy.value || billingBusy.value) return;
  if (hasUnsaved.value) pendingAction.value = 'discard';
  else emit('cancel');
}
watch(pendingAction, async (action) => {
  if (action) {
    confirmationReturnFocus = document.activeElement;
    await nextTick();
    confirmCancel.value?.focus();
  } else if (confirmationReturnFocus?.isConnected) {
    confirmationReturnFocus.focus();
  }
});
function trapConfirmationFocus(event) {
  const buttons = event.currentTarget.querySelectorAll('button');
  if ((event.shiftKey && document.activeElement === buttons[0]) || (!event.shiftKey && document.activeElement === buttons[1])) {
    event.preventDefault(); buttons[event.shiftKey ? 1 : 0].focus();
  }
}
async function confirmAction() {
  const action = pendingAction.value;
  pendingAction.value = null;
  if (action === 'discard') emit('cancel');
  else if (action === 'restore') await doRestore();
  else if (action === 'history') await clearHistory();
}

async function savePreference(key, state, value, event, apply) {
  const previous = state.value;
  if (saving.value) { if (event) event.target[event.target.type === 'checkbox' ? 'checked' : 'value'] = previous; return false; }
  saving.value = true;
  try {
    await setLocalSetting(key, typeof value === 'boolean' ? (value ? '1' : '0') : value);
    state.value = value;
    apply?.();
    showFeedback('已保存');
    return true;
  } catch (error) {
    if (event) event.target[event.target.type === 'checkbox' ? 'checked' : 'value'] = previous;
    showFeedback(`保存失败：${error.message || error}`, true);
    return false;
  } finally { saving.value = false; }
}

onMounted(loadSettings);

async function loadSettings() {
  loading.value = true;
  try {
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
  autoBackup.value = isPrefOn(await getLocalSetting("auto_backup"));
  await refreshAutoBackupNote();
  squareAccess.value = isPrefOn(await getLocalSetting("square_access"), true);
  uiLanguage.value = (await getLocalSetting("ui_language")) || "zh";
  promptBilingual.value = isPrefOn(await getLocalSetting("prompt_bilingual"), true);
  density.value = (await getLocalSetting("density")) || "comfortable";
  defaultModel.value = (await getLocalSetting("default_model")) || "";
  modelCatalog.value = (await getLocalSetting("model_catalog")) || "";
  showModelTags.value = isPrefOn(await getLocalSetting("show_model_tags"), true);
  variableHints.value = isPrefOn(await getLocalSetting("variable_hints"));
  customModels.value = (await getLocalSetting("custom_models")) || "";
  keepAuthorOnDownload.value = isPrefOn(await getLocalSetting("keep_author_on_download"));
  autoDownload.value = isPrefOn(await getLocalSetting("auto_download"));
  updateChannel.value = (await getLocalSetting("update_channel")) === "preview" ? "preview" : "stable";
  syncConflict.value = (await getLocalSetting("sync_conflict")) === "keep_local" ? "keep_local" : "newer";
  syncWifiImages.value = isPrefOn(await getLocalSetting("sync_wifi_images"));
  autoSyncQueue.value = isPrefOn(await getLocalSetting("auto_sync_queue"));
  anonymousDownloadStats.value = isPrefOn(await getLocalSetting("anonymous_download_stats"));
  httpProxy.value = (await getLocalSetting("http_proxy")) || "";
  if (props.session.loggedIn) {
    const [mine, profile, billing] = await Promise.all([
      listMyPublications().catch(() => []),
      getMe().catch(() => null),
      getBillingStatus().catch((error) => ({ note: error.message || String(error) })),
    ]);
    myPublications.value = mine;
    displayName.value = profile?.display_name ?? profile?.displayName ?? "";
    bio.value = profile?.bio ?? "";
    applyBilling(billing);
  }
  savedDrafts.value = { models: modelDraft(), shortcuts: shortcutDraft(), profile: profileDraft() };
  } catch (error) { showFeedback(`读取设置失败：${error.message || error}`, true); }
  finally { loading.value = false; }
}

async function togglePref(key, event) {
  const enabled = event.target.checked;
  const state = key === DESKTOP_PREF_KEYS.launchAtLogin ? launchAtLogin : key === DESKTOP_PREF_KEYS.minimizeToTray ? minimizeToTray : closeLauncherAfterUse;
  const previous = state.value;
  if (saving.value) { event.target.checked = previous; return; }
  saving.value = true;
  prefError.value = "";
  try {
    await saveDesktopPref(key, enabled, props.host, setLocalSetting);
    if (key === DESKTOP_PREF_KEYS.launchAtLogin) launchAtLogin.value = enabled;
    if (key === DESKTOP_PREF_KEYS.minimizeToTray) minimizeToTray.value = enabled;
    if (key === DESKTOP_PREF_KEYS.closeLauncherAfterUse) closeLauncherAfterUse.value = enabled;
    showFeedback('已保存');
  } catch (error) {
    prefError.value = error instanceof Error ? error.message : String(error);
    event.target.checked = previous;
    state.value = previous;
    showFeedback(`保存失败：${prefError.value}`, true);
  } finally { saving.value = false; }
}

async function runSyncNow() {
  syncNote.value = "";
  if (!props.session.loggedIn) {
    emit("login");
    return;
  }
  try {
    await syncLocalLibraryNow();
    await flushSyncQueue();
    syncNote.value = "已同步";
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (message.includes("登录")) {
      emit("login");
      return;
    }
    syncNote.value = message;
  }
}

async function toggleAutoDownload(event) {
  await savePreference("auto_download", autoDownload, event.target.checked, event);
}

async function saveUpdateChannel(event) {
  await savePreference("update_channel", updateChannel, event.target.value === "preview" ? "preview" : "stable", event);
}

async function saveSyncConflict(event) {
  await savePreference("sync_conflict", syncConflict, event.target.value === "keep_local" ? "keep_local" : "newer", event);
}

async function toggleSyncWifiImages(event) {
  await savePreference("sync_wifi_images", syncWifiImages, event.target.checked, event);
}

async function toggleAutoSyncQueue(event) {
  await savePreference("auto_sync_queue", autoSyncQueue, event.target.checked, event);
}

async function toggleAnonymousDownloadStats(event) {
  await savePreference("anonymous_download_stats", anonymousDownloadStats, event.target.checked, event);
}

async function saveHttpProxy() {
  const parsed = parseHttpProxy(httpProxy.value);
  if (!parsed.ok) {
    proxyError.value = parsed.error;
    return;
  }
  proxyError.value = "";
  httpProxy.value = parsed.value;
  try {
    await setLocalSetting("http_proxy", parsed.value);
  } catch (error) {
    proxyError.value = error instanceof Error ? error.message : String(error);
  }
}

async function runCheckUpdates() {
  updateNote.value = "";
  releaseNotes.value = "";
  try {
    const result = await checkForUpdates({ channel: updateChannel.value });
    releaseNotes.value = result?.notes ?? "";
    if (result?.available) {
      const version = result.version ? ` ${result.version}` : "";
      if (autoDownload.value) {
        try {
          const queued = await queueUpdateInstall({
            autoDownload: true,
            channel: updateChannel.value,
          });
          updateNote.value = queued?.queued ? "已排队安装" : `发现更新${version}`.trim();
        } catch {
          updateNote.value = "安装失败";
        }
      } else {
        updateNote.value = `发现更新${version}`.trim();
      }
    } else {
      updateNote.value = "没有可用更新";
    }
  } catch {
    updateNote.value = "检查失败";
  }
}

async function saveShortcut() {
  if (saving.value) return;
  saving.value = true;
  shortcutError.value = "";
  try {
    const invokeCombo = shortcut.value.trim() || DEFAULT_LAUNCHER_SHORTCUT;
    const createCombo = newPromptShortcut.value.trim() || DEFAULT_NEW_PROMPT_SHORTCUT;
    const pasteCombo = pasteRecentShortcut.value.trim() || DEFAULT_PASTE_RECENT_SHORTCUT;
    if (new Set([invokeCombo, createCombo, pasteCombo]).size !== 3) {
      shortcutError.value = "三项快捷键不能使用相同组合，请重新录入。";
      showFeedback(shortcutError.value, true);
      return;
    }
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
    emit('launcher-shortcut-saved', invokeCombo);
    await setLocalSetting("new_prompt_shortcut", createCombo);
    await setLocalSetting("paste_recent_shortcut", pasteCombo);
    savedDrafts.value.shortcuts = shortcutDraft();
    showFeedback('快捷键已保存');
  } catch (error) {
    shortcutError.value = error instanceof Error ? error.message : String(error);
    showFeedback(`保存失败，部分快捷键可能已生效，请重试：${shortcutError.value}`, true);
  } finally { saving.value = false; }
}

async function doExport() {
  dataError.value = "";
  try { exportText.value = await exportLocalLibrary(); }
  catch (error) { dataError.value = `导出失败：${error.message || error}`; }
}

async function doPreview() {
  dataError.value = "";
  importNote.value = "";
  preview.value = null;
  const text = importText.value;
  try {
    const result = await previewLocalImport(text);
    if (text !== importText.value) return;
    preview.value = result;
    previewedText.value = text;
  } catch (error) { dataError.value = `预览失败：${error.message || error}`; }
}

async function doApply() {
  if (importBusy.value || !preview.value || previewedText.value !== importText.value) return;
  importBusy.value = true;
  dataError.value = "";
  try {
    await applyLocalImport(previewedText.value);
    preview.value = null;
    importText.value = "";
    importNote.value = "导入完成；原有条目未被覆盖。";
    emit("imported");
  } catch (error) { dataError.value = `导入失败：${error.message || error}`; }
  finally { importBusy.value = false; }
}

async function doBackup() {
  if (dataBusy.value) return;
  dataBusy.value = true;
  dataError.value = "";
  backupPath.value = "";
  try {
    backupPath.value = await backupLocalLibrary();
  } catch (error) {
    dataError.value = error instanceof Error ? error.message : String(error);
  } finally { dataBusy.value = false; }
}

async function doRestore() {
  if (dataBusy.value || !restorePath.value.trim()) return;
  dataBusy.value = true;
  dataError.value = "";
  try {
    await restoreLocalLibrary(restorePath.value.trim());
    dataError.value = "恢复完成。快捷键、代理和系统级偏好请重启应用后生效。";
    showFeedback(dataError.value);
    await loadSettings();
    emit("imported");
  } catch (error) {
    dataError.value = error instanceof Error ? error.message : String(error);
    showFeedback(`恢复失败：${dataError.value}`, true);
  } finally { dataBusy.value = false; }
}

async function openDir() {
  dataError.value = "";
  try {
    zipPath.value = await openLibraryDir();
  } catch (error) {
    dataError.value = error instanceof Error ? error.message : String(error);
  }
}

async function doZip() {
  dataError.value = "";
  try {
    zipPath.value = await exportLibraryZip();
  } catch (error) {
    dataError.value = error instanceof Error ? error.message : String(error);
  }
}

async function toggleAutoBackup(event) {
  if (dataBusy.value) return;
  const enabled = event.target.checked;
  dataBusy.value = true;
  dataError.value = "";
  try {
    await setAutoBackup(enabled);
    autoBackup.value = enabled;
    await refreshAutoBackupNote();
  } catch (error) {
    event.target.checked = autoBackup.value;
    dataError.value = `自动备份设置失败：${error.message || error}`;
  } finally { dataBusy.value = false; }
}

async function refreshAutoBackupNote() {
  const error = await getLocalSetting("auto_backup_error");
  const path = await getLocalSetting("auto_backup_last_path");
  autoBackupNote.value = error ? `最近自动备份失败：${error}` : (path ? `最近自动备份：${path}` : "");
}

async function toggleSquareAccess(event) {
  await savePreference("square_access", squareAccess, event.target.checked, event);
}

async function toggleKeepAuthorOnDownload(event) {
  await savePreference("keep_author_on_download", keepAuthorOnDownload, event.target.checked, event);
}

async function saveAuthorProfile() {
  if (saving.value) return;
  profileNote.value = "";
  if (!props.session.loggedIn) {
    profileNote.value = "未登录不得写入";
    return;
  }
  saving.value = true;
  try {
    const saved = await putMe({ displayName: displayName.value, bio: bio.value });
    displayName.value = saved.display_name ?? saved.displayName ?? displayName.value;
    bio.value = saved.bio ?? bio.value;
    savedDrafts.value.profile = profileDraft();
    showFeedback('作者资料已保存');
  } catch (error) {
    profileNote.value = error instanceof Error ? error.message : String(error);
    showFeedback(`保存失败：${profileNote.value}`, true);
  } finally { saving.value = false; }
}

function applyBilling(payload) {
  billingPro.value = Boolean(payload?.pro);
  billingMock.value = Boolean(payload?.mock);
  billingMockPro.value = Boolean(payload?.mock_pro);
  billingNote.value = payload?.note ?? "";
}

async function runCheckout(mockOutcome) {
  if (billingBusy.value) return;
  const token = getSession().accessToken;
  if (!props.session.loggedIn) {
    emit("login");
    return;
  }
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
  if (!props.session.loggedIn) {
    emit("login");
    return;
  }
  try {
    const payload = await redeemBillingCode(redeemCode.value);
    applyBilling(payload);
  } catch (error) {
    billingNote.value = error instanceof Error ? error.message : String(error);
  }
}

async function saveTheme(event) {
  await savePreference('theme', themeChoice, event.target.value, event, () => emit('theme', themeChoice.value));
}

async function saveUiLanguage(value, event) {
  await savePreference("ui_language", uiLanguage, value, event, () => {
    document.documentElement.lang = value === "en" ? "en" : "zh-CN";
    emit("language", value);
  });
}

async function toggleBilingual(event) {
  await savePreference("prompt_bilingual", promptBilingual, event.target.checked, event);
}

async function saveDensity(value, event) {
  await savePreference("density", density, value, event, () => { document.body.dataset.density = value; });
}

async function saveModels() {
  if (saving.value) return;
  saving.value = true;
  try {
  await setLocalSetting("default_model", defaultModel.value);
  await setLocalSetting("model_catalog", modelCatalog.value);
  await setLocalSetting("show_model_tags", showModelTags.value ? "1" : "0");
  await setLocalSetting("variable_hints", variableHints.value ? "1" : "0");
  await setLocalSetting("custom_models", customModels.value);
  savedDrafts.value.models = modelDraft();
  showFeedback('模型偏好已保存');
  } catch (error) { showFeedback(`保存失败，部分偏好可能已保存，请重试：${error.message || error}`, true); }
  finally { saving.value = false; }
}

async function clearHistory() {
  if (saving.value) return;
  saving.value = true;
  try { await clearLocalPromptUse(); emit("history-cleared"); showFeedback('使用历史已清除，提示词未删除'); }
  catch (error) { showFeedback(`清除失败：${error.message || error}`, true); }
  finally { saving.value = false; }
}
</script>
