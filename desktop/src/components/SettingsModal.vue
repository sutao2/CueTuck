<template>
  <section class="settings-page" data-testid="settings-page" aria-label="设置" @keydown.esc="onEscape">
      <div class="settings-window-drag" data-tauri-drag-region aria-hidden="true"></div>
      <div class="settings-body" :inert="pendingAction ? '' : undefined">
        <nav class="settings-nav" aria-labelledby="settings-title">
          <button ref="returnButton" type="button" class="settings-return" aria-label="返回应用" :disabled="saving || loading || dataBusy || importBusy || billingBusy || logoutBusy || syncBusy" @click="requestClose"><span aria-hidden="true">←</span> 返回应用</button>
          <label class="settings-search"><AppIcon name="search" /><input v-model="settingsQuery" type="search" aria-label="搜索设置" placeholder="搜索设置…" @keydown.esc.stop="clearSearchOrReturn" /></label>
          <h2 id="settings-title">{{ uiText(uiLanguage, "settings") }}</h2>
          <button
            v-for="page in filteredPages"
            :key="page.id"
            type="button"
            :class="{ active: current === page.id }"
            :data-settings-page="page.id"
            :aria-current="current === page.id ? 'page' : undefined"
            @click="current = page.id"
          >
            <AppIcon :name="page.icon" /><span>{{ page.label }}</span>
          </button>
          <p v-if="!filteredPages.length" class="settings-no-results" role="status">没有匹配的设置</p>
        </nav>
        <main :key="current" class="settings-content" :aria-label="pages.find(page => page.id === current)?.label">
          <fieldset class="settings-fields" :disabled="saving || loading">
          <section v-if="current === 'general'">
            <h3>常规</h3>
            <p>管理应用启动、托盘和快捷窗口的使用偏好。</p>
            <div class="settings-group">
            <p v-if="prefError" data-testid="pref-error">{{ prefError }}</p>
            <label class="setting-row">
              <span class="setting-copy"><strong>开机启动</strong><small>登录系统后自动打开唤词。</small></span>
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
            </div>
            <h4>启动器</h4>
            <p>仅影响独立快捷窗口，保存后下次唤起生效。</p>
            <div class="settings-group">
              <label v-for="option in launcherSettingRows" :key="option.key" class="setting-row">
                <span class="setting-copy"><strong>{{ option.label }}</strong><small>{{ option.description }}</small></span>
                <select :data-testid="`launcher-${option.key}`" :value="launcherPreferences[option.key]" @change="changeLauncherPreference(option.key, $event)">
                  <option v-for="choice in option.choices" :key="choice.value" :value="choice.value">{{ choice.label }}</option>
                </select>
              </label>
              <div class="setting-row">
                <span class="setting-copy"><strong>恢复启动器默认设置</strong><small>仅恢复上面四项，不修改快捷键或使用后关闭。</small></span>
                <button type="button" class="button ghost-button" data-testid="reset-launcher-preferences" @click="saveLauncherPreferences({ ...DEFAULT_LAUNCHER_PREFERENCES })">恢复默认</button>
              </div>
              <div class="setting-row">
                <span class="setting-copy"><strong>启动器快捷键</strong><small>唤起、新建与粘贴最近使用的组合键。</small></span>
                <button type="button" class="button ghost-button" @click="current = 'shortcuts'">前往快捷键</button>
              </div>
            </div>
          </section>
          <section v-else-if="current === 'account'" class="account-page">
            <h3>账号与广场</h3>
            <p>管理你的身份，分享你的创作。</p>
            <p v-if="logoutError" role="alert">{{ logoutError }}</p>
            <div class="account-overview" data-testid="account-overview">
              <div class="account-avatar" aria-hidden="true">{{ accountInitial }}</div>
              <div class="account-identity">
                <span class="account-eyebrow">{{ session.loggedIn ? '当前账号' : '本地工作空间' }}</span>
                <h4 data-testid="account-name">{{ accountName }}</h4>
                <span class="account-email" data-testid="current-account">{{ session.loggedIn ? session.email : '未登录' }}</span>
              </div>
              <div class="account-session-actions">
                <span v-if="session.loggedIn" class="account-badge">已登录</span>
                <button v-if="!session.loggedIn" type="button" class="button primary-button" data-testid="settings-login" @click="$emit('login')">登录</button>
                <button v-else type="button" class="button ghost-button" data-testid="settings-logout" :disabled="logoutBusy" @click="$emit('logout')">{{ logoutBusy ? '正在退出…' : '退出登录' }}</button>
              </div>
            </div>
            <div class="account-grid">
              <div class="account-card setting-block account-profile-card">
                <header class="account-card-heading"><span class="account-section-icon"><AppIcon name="user" /></span><div><h4>公开资料</h4><p>用于作者主页，让广场里的创作有你的名字。</p></div></header>
                <form class="author-profile" @submit.prevent="saveAuthorProfile">
                  <label class="profile-field"><span>显示名</span>
                    <input data-testid="author-display-name" :disabled="!session.loggedIn" v-model="displayName" placeholder="你希望大家怎么称呼你？" @input="profileNote = ''">
                  </label>
                  <label class="profile-field"><span>简介 <small>选填</small></span>
                    <textarea data-testid="author-bio" :disabled="!session.loggedIn" v-model="bio" rows="5" placeholder="介绍你的创作方向，或你擅长的领域…" @input="profileNote = ''"></textarea>
                  </label>
                  <p class="account-form-hint">{{ session.loggedIn ? '显示名与简介会公开展示，登录邮箱不会因编辑资料而改变。' : '登录后即可编辑公开资料。' }}</p>
                  <div class="account-save-row">
                    <button type="submit" class="button primary-button" data-testid="save-author-profile" :disabled="!session.loggedIn" @click.prevent="saveAuthorProfile">{{ saving ? '正在保存…' : '保存资料' }}</button>
                    <small role="status" data-testid="author-profile-note">{{ profileNote || (profileDirty ? '有未保存的修改' : '修改后点击保存') }}</small>
                  </div>
                </form>
              </div>
              <div class="account-side">
                <div class="account-card account-publications">
                  <header class="account-card-heading"><span class="account-section-icon"><AppIcon name="globe" /></span><div><h4>我的发布</h4><p>查看投稿、审核进度与已公开的作品。</p></div></header>
                  <button type="button" class="button ghost-button" data-testid="settings-publications" @click="$emit('publications')">查看我的发布 <span aria-hidden="true">↗</span></button>
                </div>
                <div class="account-card setting-block account-billing-card">
                  <div class="account-plan-heading"><h4>订阅与权益</h4><span v-if="session.loggedIn && billingMock" class="account-test-badge">Mock · 测试环境</span></div>
                  <strong class="account-plan" data-testid="billing-pro">{{ session.loggedIn ? (billingPro ? 'Pro' : '未订阅') : '未登录' }}</strong>
                  <p class="account-form-hint">{{ session.loggedIn ? '本地创作随时可用，订阅状态以当前账号为准。' : '登录后查看订阅与兑换权益。' }}</p>
                  <small v-if="session.loggedIn && billingMock" class="account-mock-status" data-testid="billing-mock">Mock · {{ billingMockPro ? '模拟 Pro' : '模拟未订阅' }}（不扣款，不改变真实权益）</small>
                  <small v-if="!billingMock && billingNote" class="account-mock-status" role="status" data-testid="billing-note">{{ billingNote }}</small>
                  <details class="account-billing-details">
                    <summary>{{ billingMock ? '模拟测试与测试码' : '兑换码与支付' }}</summary>
                    <div class="author-profile">
                      <small v-if="billingMock && billingNote" role="status" data-testid="billing-note">{{ billingNote }}</small>
                      <div v-if="session.loggedIn && billingMock" class="account-test-actions">
                        <button v-for="(label, outcome) in { success: '模拟成功', failure: '模拟失败', cancel: '模拟取消', reset: '重置模拟' }" :key="outcome" type="button" class="button ghost-button" :data-testid="`billing-mock-${outcome}`" :disabled="billingBusy" @click="runCheckout(outcome)">{{ label }}</button>
                      </div>
                      <label class="profile-field"><span>{{ billingMock ? 'Mock 测试码（仅模拟权益）' : '兑换码' }}</span>
                        <input data-testid="billing-redeem-code" :disabled="!session.loggedIn || billingBusy" v-model="redeemCode" placeholder="输入兑换码">
                      </label>
                      <div class="account-test-actions">
                        <button type="button" class="button ghost-button" data-testid="billing-redeem" :disabled="!session.loggedIn || billingBusy || (billingMock && !/^TEST-[A-F0-9]{32}$/i.test(redeemCode.trim()))" @click="runRedeem">{{ billingMock ? '兑换测试码' : '兑换' }}</button>
                        <button v-if="!billingMock" type="button" class="button primary-button" data-testid="billing-checkout" :disabled="!session.loggedIn || billingBusy" @click="runCheckout">前往支付</button>
                      </div>
                    </div>
                  </details>
                </div>
              </div>
            </div>
            <div class="account-card account-preferences">
              <label class="setting-row">
                <span class="setting-copy"><strong>下载时保留作者信息</strong><small>新下载的本地副本展示原作者，不改变提示词正文。</small></span>
                <input type="checkbox" data-testid="keep-author-on-download" :checked="keepAuthorOnDownload" @change="toggleKeepAuthorOnDownload">
              </label>
            </div>
          </section>
          <section v-else-if="current === 'shortcuts'">
            <h3>快捷键</h3>
            <p>登记全局组合以唤起独立启动器。与系统冲突时会提示，不会静默失效。</p>
            <div class="settings-group">
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
            </div>
          </section>
          <section v-else-if="current === 'sync'" data-testid="settings-unavailable">
            <h3>同步</h3>
            <p>已登录可立即同步个人库。启动器始终只读本机；MCP 广场工具需在接入配置中另行启用。</p>
            <div class="settings-group">
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
              <span class="setting-copy"><strong>仅在 Wi-Fi 下同步图片与附件</strong><small>无法判定或非 Wi-Fi 时，封面和本次勾选的附件均延后；标题与正文照常同步。</small></span>
              <input
                type="checkbox"
                data-testid="sync-wifi-images"
                :checked="syncWifiImages"
                @change="toggleSyncWifiImages"
              >
            </label>
            <div class="setting-row" data-testid="sync-conflict">
              <span class="setting-copy"><strong>冲突处理</strong><small>默认采用较新正文，附件仅补齐合并。保留本地时跳过已有提示词的正文与附件下载，远端独有条目仍写入。</small></span>
              <select data-testid="sync-conflict-strategy" :value="syncConflict" @change="saveSyncConflict">
                <option value="newer">较新者胜</option>
                <option value="keep_local">保留本地</option>
              </select>
            </div>
            <div class="setting-row">
              <label class="setting-copy" for="include-sync-assets"><strong>本次包含私有附件</strong><small>将本机库附件上传到 {{ session.email || '当前账号' }}，并补齐账号库文件，不公开。合并保留两端已有附件，不同步附件删除；每次需重新勾选。</small></label>
              <input id="include-sync-assets" v-model="syncIncludeAssets" :disabled="syncBusy" type="checkbox" data-testid="sync-include-assets">
            </div>
            <div class="setting-row">
              <span class="setting-copy"><strong>立即同步</strong><small>已登录时推拉账号库。未登录打开登录，不会假装已同步。</small></span>
              <button type="button" class="button ghost-button" data-testid="sync-now" :disabled="syncBusy" @click="runSyncNow()">{{ syncBusy ? '正在同步…' : '立即同步' }}</button>
            </div>
            <p v-if="syncNote" role="status" data-testid="sync-note">{{ syncNote }}</p>
            <button v-if="syncQueuePending" type="button" class="button ghost-button" data-testid="retry-sync-queue" :disabled="syncBusy" @click="runSyncNow(true)">重试发送队列</button>
            </div>
          </section>
          <section v-else-if="current === 'models'">
            <h3>AI 与模型</h3>
            <p>管理本机模型标签，以及独立的启动器 AI 优化配置。</p>
            <div class="settings-group">
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
            </div>
          </section>
          <section v-else-if="current === 'data'">
            <h3>数据与备份</h3>
            <p>管理本机资料、导入导出与数据恢复。</p>
            <div class="settings-group">
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
            <div class="import-drop" data-testid="import-drop" @dragover.prevent @drop.prevent="dropImport">
              <strong>导入提示词文件</strong><p>选择或拖入 JSON 文件，预览后再确认导入。</p>
              <label class="button ghost-button file-choice">选择 JSON 文件<input type="file" accept=".json,application/json" :disabled="importBusy" data-testid="import-file" @change="chooseImport" /></label>
              <p v-if="importFilename" role="status">{{ importBusy ? '正在读取：' : '已选择：' }}{{ importFilename }}</p>
            </div>
            <details class="advanced-import"><summary>高级：粘贴 JSON</summary>
            <label class="field">
              <span>导入 JSON</span>
              <textarea v-model="importText" :disabled="importBusy" rows="5" placeholder='{"prompts":[{"title":"一","content":"a"}]}' @input="preview = null; importFilename = ''"></textarea>
            </label>
            </details>
            <div class="modal-actions">
              <button type="button" class="button ghost-button" :disabled="importBusy" @click="doPreview">预览</button>
              <button type="button" class="button primary-button" :disabled="importBusy || !preview" @click="doApply">确认导入</button>
            </div>
            <p v-if="preview" data-testid="import-preview">
              将导入 {{ preview.prompt_count }} 条提示词、{{ preview.collection_count }} 个合集。确认前不会写入。
            </p>
            <p v-if="importNote" role="status">{{ importNote }}</p>
            <div class="restore-choice"><button type="button" class="button ghost-button" data-testid="choose-restore" :disabled="dataBusy || !usesSystemKeychain()" @click="chooseRestore">选择备份文件</button><small v-if="!usesSystemKeychain()">文件恢复需要桌面应用</small></div>
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
            </div>
          </section>
          <section v-else-if="current === 'network'">
            <h3>网络与代理</h3>
            <p>控制联网范围，以及桌面端使用的代理。</p>
            <div class="settings-group">
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
            </div>
            <McpSettings />
          </section>
          <section v-else-if="current === 'appearance'">
            <h3>外观</h3>
            <p>选择适合你的主题、语言与内容密度。</p>
            <div class="theme-choices" role="group" aria-label="主题">
              <button v-for="option in [{id:'system',label:'跟随系统'},{id:'light',label:'浅色'},{id:'dark',label:'深色'}]" :key="option.id" type="button" class="theme-choice" :data-theme-choice="option.id" :aria-pressed="themeChoice === option.id" @click="chooseTheme(option.id)">
                <span class="theme-preview" :class="`preview-${option.id}`" aria-hidden="true"><span class="preview-sidebar"></span><span class="preview-paper"><i></i><i></i><i></i></span></span>
                <span>{{ option.label }}</span>
              </button>
            </div>
            <div class="settings-group">
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
            </div>
          </section>
          <section v-else-if="current === 'privacy'">
            <h3>隐私与安全</h3>
            <p>了解数据的保存方式，管理统计与使用记录。</p>
            <div class="settings-group">
            <div class="setting-row">
              <span class="setting-copy"><strong>本地提示词默认不上传</strong><small>未点发布不得把本地正文送出。</small></span>
              <span class="setting-control">始终生效</span>
            </div>
            <label class="setting-row" data-testid="anonymous-download-stats-row">
              <span class="setting-copy"><strong>匿名下载统计</strong><small>默认开启，可随时关闭。成功下载只上报条目 id，不含账号、正文或标题。关闭时不请求。统计失败不影响下载。</small></span>
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
            </div>
          </section>
          <section v-else-if="current === 'updates'" data-testid="settings-updates">
            <h3>更新</h3>
            <p>管理版本、更新通道与发行说明。</p>
            <div class="settings-group">
            <div class="setting-row">
              <span class="setting-copy"><strong>当前版本</strong><small>桌面包 {{ appVersion }}，与本机构建一致。</small></span>
              <button type="button" class="button ghost-button" data-testid="check-updates" :disabled="['checking','downloading','verifying','ready','installing'].includes(updateState.phase)" @click="runCheckUpdates">检查更新</button>
            </div>
            <label class="setting-row">
              <span class="setting-copy"><strong>自动下载更新</strong><small>发现新版本后自动下载并验证，安装前由你确认。</small></span>
              <input
                type="checkbox"
                data-testid="auto-download"
                :checked="autoDownload"
                @change="toggleAutoDownload"
              >
            </label>
            <label class="setting-row">
              <span class="setting-copy"><strong>更新通道</strong><small>稳定版只用正式发行，预览版只用预发行。</small></span>
              <select data-testid="update-channel" :disabled="['checking','downloading','verifying','ready','installing'].includes(updateState.phase)" :value="updateChannel" @change="saveUpdateChannel">
                <option value="stable">稳定版</option>
                <option value="preview">预览版</option>
              </select>
            </label>
            <div class="setting-row">
              <span class="setting-copy"><strong>发行说明</strong><small>随检查更新展示，不来自应用商店。</small></span>
              <span class="setting-control">GitHub Releases</span>
            </div>
            <UpdateStatus :channel="updateChannel" :dirty="hasUnsaved" />
            </div>
          </section>
          <LauncherAiSettings v-if="aiVisited" v-show="current === 'models'" @dirty="aiDirty = $event" @busy="aiBusy = $event" />
          </fieldset>
        </main>
      </div>
      <footer class="settings-feedback" :class="{ error: feedbackError }" role="status" aria-live="polite" data-testid="settings-feedback">
        {{ loading ? '正在读取设置…' : saving ? '正在保存…' : feedback || (current === 'sync' ? '附件授权仅用于本次同步，不保存；其余偏好即时保存。' : '开关与选择项即时保存；有保存按钮的表单需手动保存。') }}
      </footer>
      <div v-if="pendingAction" class="settings-confirm-layer">
        <section class="settings-confirm" role="alertdialog" aria-modal="true" aria-labelledby="settings-confirm-title" aria-describedby="settings-confirm-copy" @keydown.tab="trapConfirmationFocus">
          <h3 id="settings-confirm-title">{{ pendingAction === 'discard' ? '放弃未保存的修改？' : pendingAction === 'restore' ? '恢复数据库？' : '清除使用历史？' }}</h3>
          <p id="settings-confirm-copy">{{ pendingAction === 'discard' ? '尚未保存的表单和导入文本将被丢弃，已经保存的设置不受影响。' : pendingAction === 'restore' ? `将使用 ${restorePath} 替换当前库。请先备份当前数据，此操作不是合并导入。` : '将清除最近使用记录与使用次数，不删除提示词正文。' }}</p>
          <div class="modal-actions">
            <button ref="confirmCancel" type="button" class="button" data-testid="cancel-settings-action" @click="pendingAction = null; emit('stay')">{{ pendingAction === 'discard' ? '继续编辑' : '取消' }}</button>
            <button type="button" class="button danger-button" data-testid="confirm-settings-action" @click="confirmAction">{{ pendingAction === 'discard' ? '放弃修改' : '确认执行' }}</button>
          </div>
        </section>
      </div>
  </section>
</template>

<script setup>
import McpSettings from './McpSettings.vue';
import AppIcon from "./AppIcon.vue";
import LauncherAiSettings from './LauncherAiSettings.vue';
import { DEFAULT_LAUNCHER_PREFERENCES, LAUNCHER_PREFERENCES_KEY, readLauncherPreferences } from '../platform/launcherPreferences.js';
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
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
import { invokeCommand } from '../platform/tauri.js';
import pkg from "../../package.json";
import { DESKTOP_PREF_KEYS, isPrefOn, saveDesktopPref } from "../platform/desktopPrefs.js";

import { getMe, putMe, getSession } from "../platform/session.js";
import {
  getBillingStatus,
  redeemBillingCode,
  startBillingCheckout,
} from "../platform/billing.js";
import { syncLocalLibraryNow } from "../platform/librarySync.js";
import { parseHttpProxy } from "../platform/httpProxy.js";
import { flushSyncQueue } from "../platform/syncQueue.js";
import { updateState, refreshUpdate, defaultUpdateChannel } from "../platform/updates.js";
import UpdateStatus from "./UpdateStatus.vue";

const props = defineProps({
  theme: { type: String, default: "light" },
  host: { type: String, default: "macos" },
  session: { type: Object, default: () => ({ loggedIn: false, email: "" }) },
  language: { type: String, default: "zh" },
  initialPage: { type: String, default: 'general' },
  logoutBusy: { type: Boolean, default: false },
  logoutError: { type: String, default: '' },
});
const emit = defineEmits(["cancel", "stay", "theme", "imported", "login", "logout", "history-cleared", "language", "launcher-shortcut-saved", "publications"]);

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
const current = ref(pages.value.some(page => page.id === props.initialPage) ? props.initialPage : 'general');
const settingsQuery = ref(''), returnButton = ref(null);
const searchKeywords = {
  general: '启动器 大小 尺寸 位置 字号 搜索结果 托盘 窗口 startup tray launcher size position font', account: '登录 邮箱 资料 订阅 账单 兑换 作者 login profile billing',
  shortcuts: '快捷键 按键 启动器 录入 keyboard launcher', sync: '同步 冲突 wifi 云 sync',
  models: '模型 默认 标签 变量 建议 model ai', data: '数据 备份 恢复 导入 导出 backup restore import export',
  network: '代理 广场 网络 智能体 MCP 接入 proxy network', appearance: '外观 主题 深色 浅色 语言 密度 theme language',
  privacy: '隐私 统计 历史 安全 钥匙串 privacy history', updates: '更新 版本 下载 通道 update version',
};
const filteredPages = computed(() => {
  const terms = settingsQuery.value.trim().toLowerCase().split(/\s+/);
  return pages.value.filter(page => terms.every(term => `${page.label} ${searchKeywords[page.id]}`.toLowerCase().includes(term)));
});
let previousFocus;
onMounted(() => { previousFocus = document.activeElement; returnButton.value?.focus(); });
onUnmounted(() => nextTick(() => { if (previousFocus?.isConnected) previousFocus.focus(); }));
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
const autoDownload = ref(false);
const updateChannel = ref(defaultUpdateChannel);
const syncConflict = ref("newer");
const syncWifiImages = ref(false);
const syncIncludeAssets = ref(false);
const syncBusy = ref(false);
const syncQueuePending = ref(false);
let syncLibrarySummary = '';
watch(() => props.session.email, () => { syncIncludeAssets.value = false; syncNote.value = ''; syncQueuePending.value = false; syncLibrarySummary = ''; });
const autoSyncQueue = ref(false);
const anonymousDownloadStats = ref(true);
const httpProxy = ref("");
const proxyError = ref("");
const appVersion = pkg.version;
const prefError = ref("");
const launchAtLogin = ref(false);
const minimizeToTray = ref(false);
const closeLauncherAfterUse = ref(true);
const launcherPreferences = ref({ ...DEFAULT_LAUNCHER_PREFERENCES });
const launcherSettingRows = [
  { key: 'size', label: '窗口大小', description: '搜索结果与填写页保持同样大小，小屏会限制在可用区域。', choices: [{ value: 'compact', label: '紧凑 · 620 × 420' }, { value: 'standard', label: '标准 · 680 × 500' }, { value: 'large', label: '宽敞 · 760 × 560' }] },
  { key: 'position', label: '默认位置', description: '每次唤起的位置；本次拖动后，输入或清空不会跳位。', choices: [{ value: 'upper', label: '屏幕偏上' }, { value: 'center', label: '屏幕居中' }] },
  { key: 'fontSize', label: '填写与预览字号', description: '只调整启动器里的变量输入和提示词正文。', choices: [12, 14, 16].map(value => ({ value, label: `${value} px` })) },
  { key: 'resultLimit', label: '搜索结果数量', description: '最多展示的本地搜索结果，方向键可滚动选择。', choices: [10, 20, 50].map(value => ({ value, label: `${value} 条` })) },
];
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

const billingPro = ref(false);
const billingMock = ref(false);
const billingMockPro = ref(false);
const billingBusy = ref(false);
const billingNote = ref("");
const redeemCode = ref("");
const displayName = ref("");
const bio = ref("");
const profileNote = ref("");
const savedProfileName = ref("");
const accountName = computed(() => props.session.loggedIn ? (savedProfileName.value.trim() || props.session.email?.split("@")[0] || "我的账号") : "本地访客");
const accountInitial = computed(() => Array.from(accountName.value)[0]?.toUpperCase() || "P");
const saving = ref(false);
const loading = ref(true);
const feedback = ref("");
const feedbackError = ref(false);
const pendingAction = ref(null);
const confirmCancel = ref(null);
const aiVisited = ref(current.value === 'models');
watch(current, value => { if(value === 'models') aiVisited.value = true; });
const aiDirty = ref(false), aiBusy = ref(false);
const savedDrafts = ref({});
let confirmationReturnFocus = null;
const modelDraft = () => JSON.stringify([defaultModel.value, modelCatalog.value, customModels.value, showModelTags.value, variableHints.value]);
const shortcutDraft = () => JSON.stringify([shortcut.value, newPromptShortcut.value, pasteRecentShortcut.value]);
const profileDraft = () => JSON.stringify([displayName.value, bio.value]);
const profileDirty = computed(() => !loading.value && profileDraft() !== savedDrafts.value.profile);
const hasUnsaved = computed(() => !loading.value && (
  aiDirty.value || modelDraft() !== savedDrafts.value.models || shortcutDraft() !== savedDrafts.value.shortcuts ||
  profileDraft() !== savedDrafts.value.profile || Boolean(importText.value.trim())
));

function showFeedback(message, error = false) { feedback.value = message; feedbackError.value = error; }
function onEscape(event) {
  if (event.isComposing || document.querySelector('[data-testid="login-modal"]')) return;
  event.preventDefault();
  event.stopPropagation();
  if (pendingAction.value) { pendingAction.value = null; emit('stay'); }
  else requestClose();
}
function clearSearchOrReturn(event) {
  if (event.isComposing) return;
  if (settingsQuery.value) settingsQuery.value = '';
  else requestClose();
}
const navigationBusy = computed(() => aiBusy.value || saving.value || loading.value || dataBusy.value || importBusy.value || billingBusy.value || syncBusy.value);
function requestClose() {
  if (navigationBusy.value) return;
  if (hasUnsaved.value) pendingAction.value = 'discard';
  else emit('cancel');
}
defineExpose({ requestClose, busy: navigationBusy });
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

async function saveLauncherPreferences(next, event, key) {
  if (saving.value) return;
  saving.value = true;
  try {
    await setLocalSetting(LAUNCHER_PREFERENCES_KEY, JSON.stringify(next));
    launcherPreferences.value = next;
    showFeedback('已保存，下次唤起启动器生效');
  } catch (error) {
    if (event) event.target.value = launcherPreferences.value[key];
    showFeedback(`保存失败：${error.message || error}`, true);
  } finally { saving.value = false; }
}
function changeLauncherPreference(key, event) {
  const value = ['fontSize', 'resultLimit'].includes(key) ? Number(event.target.value) : event.target.value;
  return saveLauncherPreferences({ ...launcherPreferences.value, [key]: value }, event, key);
}

onMounted(async () => { await loadSettings(); await nextTick(); returnButton.value?.focus(); });

async function loadSettings() {
  loading.value = true;
  try {
  const stored = await getLocalSetting("launcher_shortcut");
  launcherPreferences.value = await readLauncherPreferences();
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
  updateChannel.value = (await getLocalSetting("update_channel")) || defaultUpdateChannel;
  syncConflict.value = (await getLocalSetting("sync_conflict")) === "keep_local" ? "keep_local" : "newer";
  syncWifiImages.value = isPrefOn(await getLocalSetting("sync_wifi_images"));
  autoSyncQueue.value = isPrefOn(await getLocalSetting("auto_sync_queue"));
  anonymousDownloadStats.value = (await getLocalSetting("anonymous_download_stats")) !== "0";
  httpProxy.value = (await getLocalSetting("http_proxy")) || "";
  if (props.session.loggedIn) {
    const [profile, billing] = await Promise.all([
      getMe().catch(() => null),
      getBillingStatus().catch((error) => ({ note: error.message || String(error) })),
    ]);
    displayName.value = profile?.display_name ?? profile?.displayName ?? "";
    bio.value = profile?.bio ?? "";
    savedProfileName.value = displayName.value;
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

async function runSyncNow(queueOnly = false) {
  if (syncBusy.value) return;
  syncNote.value = "";
  if (!props.session.loggedIn) {
    emit("login");
    return;
  }
  syncBusy.value = true;
  const account = getSession();
  const sameAccount = () => getSession().accessToken === account.accessToken && props.session.email === account.email;
  const includeAssets = syncIncludeAssets.value;
  if (!queueOnly) syncLibrarySummary = '';
  try {
    if (!queueOnly) {
      const result = await syncLocalLibraryNow({ includeAssets, onProgress: text => { if (sameAccount()) syncNote.value = text; } });
      if (!sameAccount()) throw Error('登录状态已改变，请重新同步');
      syncLibrarySummary = result.attachmentsDeferred ? '正文已同步；当前网络不是已确认的 Wi-Fi，附件已延后' : includeAssets ? '已同步，私有附件已校验并补齐' : '个人库已同步';
      if (includeAssets && !result.attachmentsDeferred && syncConflict.value === 'keep_local') syncLibrarySummary = '已同步；保留本地策略已跳过已有提示词的附件下载';
    }
    syncQueuePending.value = true;
    syncNote.value = `${syncLibrarySummary}；正在发送收藏与发布草稿队列…`;
    const remaining = await flushSyncQueue();
    if (!sameAccount()) throw Error('登录状态已改变，请重新同步');
    const count = remaining.filter(job => job.email === account.email).length;
    syncQueuePending.value = count > 0;
    syncNote.value = `${syncLibrarySummary}；${count ? `队列仍有 ${count} 项未送达，请重试` : '队列已处理完成'}`;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    if (!sameAccount()) {
      syncNote.value = '登录状态已改变，请在当前账号重新同步';
      syncQueuePending.value = false;
      return;
    }
    syncNote.value = `${syncLibrarySummary ? `${syncLibrarySummary}；队列处理失败：` : '同步未完成：'}${message}`;
  } finally { syncBusy.value = false; syncIncludeAssets.value = false; }
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
  await refreshUpdate({ channel: updateChannel.value, autoDownload: autoDownload.value });
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

const importFilename = ref('');
async function chooseImport(event) {
  const file = event.target.files?.[0];
  event.target.value = '';
  if (file) await readImportFile(file);
}
async function dropImport(event) {
  if (importBusy.value) return;
  const files = [...(event.dataTransfer?.files || [])];
  if (files.length !== 1) { dataError.value = '请一次选择一个 JSON 文件。'; return; }
  await readImportFile(files[0]);
}
async function readImportFile(file) {
  if (importBusy.value) return;
  if (!/\.json$/i.test(file.name)) { dataError.value = '请选择 JSON 文件。'; return; }
  importBusy.value = true; dataError.value = ''; preview.value = null;
  try {
    const text = await new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result || ''));
      reader.onerror = () => reject(reader.error || Error('无法读取文件'));
      reader.readAsText(file);
    });
    JSON.parse(text);
    importText.value = text; importFilename.value = file.name;
    await doPreview();
  } catch (error) { dataError.value = `读取文件失败：${error.message || error}`; }
  finally { importBusy.value = false; }
}
async function chooseRestore() {
  if (dataBusy.value) return;
  dataBusy.value = true; dataError.value = '';
  try {
    const path = await invokeCommand('choose_library_backup');
    if (path) restorePath.value = path;
  } catch (error) { dataError.value = `选择文件失败：${error.message || error}`; }
  finally { dataBusy.value = false; }
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
    importText.value = ""; importFilename.value = '';
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
    savedProfileName.value = displayName.value;
    profileNote.value = "资料已保存";
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
  if (billingBusy.value) return;
  if (!props.session.loggedIn) {
    emit("login");
    return;
  }
  billingBusy.value=true;
  try {
    const payload = await redeemBillingCode(redeemCode.value,{mock:billingMock.value});
    applyBilling(payload);
  } catch (error) {
    billingNote.value = error instanceof Error ? error.message : String(error);
  } finally { billingBusy.value=false; }
}

async function chooseTheme(value) {
  await savePreference('theme', themeChoice, value, null, () => emit('theme', themeChoice.value));
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

<style scoped>
.account-overview { display: flex; align-items: center; gap: 20px; padding: 26px; margin: 28px 0 20px; border: 1px solid var(--line); border-radius: 16px; background: linear-gradient(115deg, var(--sidebar), var(--surface)); }
.account-avatar { display: grid; place-items: center; flex: 0 0 64px; height: 64px; border: 1px solid var(--line); border-radius: 20px; background: var(--surface); font: 550 28px var(--font-display); }
.account-identity { min-width: 0; flex: 1; }
.account-eyebrow { font-size: 11px; color: var(--muted); }
.account-identity h4 { margin: 5px 0 6px; font-size: 23px; font-weight: 550; }
.account-email { font-size: 12px; color: var(--muted); overflow-wrap: anywhere; }
.account-session-actions { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; }
.account-badge, .account-test-badge { display: inline-flex; align-items: center; gap: 6px; font-size: 11px; color: var(--muted); }
.account-badge::before { content: ''; width: 6px; height: 6px; border-radius: 50%; background: #469775; }
.account-grid { display: grid; grid-template-columns: minmax(0, 1.5fr) minmax(240px, 1fr); align-items: start; gap: 20px; }
.account-card { min-width: 0; padding: 24px; border: 1px solid var(--line); border-radius: 14px; background: var(--surface); }
.account-card h4 { margin: 0; font-size: 14px; font-weight: 550; }
.account-card-heading { display: flex; align-items: flex-start; gap: 12px; margin-bottom: 24px; }
.account-card-heading p { color: var(--muted); font-size: 12px; line-height: 1.7; margin: 6px 0 0; }
.account-section-icon { display: grid; place-items: center; flex: 0 0 30px; height: 30px; border-radius: 9px; background: var(--sidebar); }
.account-section-icon :deep(svg) { width: 16px; height: 16px; }
.account-profile-card .author-profile { gap: 18px; }
.account-profile-card textarea { min-height: 132px; line-height: 1.7; }
.account-page .profile-field > span { font-size: 12px; }
.profile-field small { margin-left: 6px; color: var(--muted); font-weight: 400; }
.account-form-hint { margin: 0; color: var(--muted); font-size: 11px; line-height: 1.7; }
.account-save-row { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; padding-top: 2px; }
.account-save-row small { font-size: 11px; color: var(--muted); }
.account-side { display: grid; gap: 20px; min-width: 0; }
.account-publications .button { display: flex; width: 100%; justify-content: space-between; }
.account-plan-heading { display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 10px; }
.account-test-badge { padding: 3px 7px; border-radius: 5px; background: var(--sidebar); }
.account-plan { display: block; margin: 18px 0 8px; font-size: 22px; font-weight: 550; }
.account-mock-status { display: block; margin-top: 12px; font-size: 11px; color: var(--muted); line-height: 1.7; }
.account-billing-details { border-top: 1px solid var(--line); margin-top: 18px; padding-top: 14px; }
.account-billing-details summary { cursor: pointer; font-size: 12px; color: var(--muted); }
.account-billing-details summary:focus-visible { outline: 2px solid var(--text); outline-offset: 4px; border-radius: 3px; }
.account-billing-details[open] summary { margin-bottom: 16px; }
.account-billing-details small { font-size: 11px; color: var(--muted); line-height: 1.7; }
.account-test-actions { display: flex; gap: 8px; flex-wrap: wrap; }
.account-preferences { margin-top: 20px; padding: 4px 24px; }
.account-preferences .setting-row { border-bottom: 0; }
@media (max-width: 1050px) {
  .account-grid { grid-template-columns: minmax(0, 1fr); }
  .account-overview { flex-wrap: wrap; }
  .account-session-actions { margin-left: 84px; }
}
@media (max-width: 640px) {
  .account-overview, .account-card { padding: 18px; }
  .account-avatar { flex-basis: 44px; height: 44px; font-size: 22px; border-radius: 14px; }
  .account-session-actions { margin-left: 64px; }
}
.import-drop { border: 1px dashed var(--line); padding: 20px; border-radius: 10px; margin: 20px 0; font-size: 13px; }
.import-drop p { color: var(--muted); font-size: 12px; }
.file-choice { position: relative; overflow: hidden; display: inline-flex; cursor: pointer; }
.file-choice input { position: absolute; inset: 0; opacity: 0; cursor: pointer; width: 100%; height: 100%; }
.file-choice:focus-within { outline: 2px solid var(--text); outline-offset: 3px; }
.advanced-import { margin: 14px 0; }
.advanced-import summary { cursor: pointer; color: var(--muted); font-size: 12px; margin-bottom: 12px; }
.restore-choice { display: flex; align-items: center; flex-wrap: wrap; gap: 12px; margin: 24px 0 16px; }
.restore-choice small { color: var(--muted); }
</style>
