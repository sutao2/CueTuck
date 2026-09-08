<template>
  <section class="mcp-settings" aria-label="智能体 MCP 接入">
    <h4>智能体 MCP 接入</h4>
    <p>让可信智能体只读搜索、读取和填写本地提示词。宿主可能将返回内容发送给模型提供商，请勿接入不可信宿主。</p>
    <p>MCP 是独立程序，不包含在桌面安装包中。先在仓库根目录执行：</p>
    <pre>cargo build --manifest-path mcp/Cargo.toml --release --locked</pre>
    <label class="field"><span>MCP 程序绝对路径</span><input v-model="executable" data-testid="mcp-executable" placeholder="/绝对路径/promptark-mcp" :disabled="busy" /></label>
    <label class="setting-row"><span class="setting-copy"><strong>此配置启用广场工具</strong><small>默认关闭。本地工具始终离线；开启后，独立广场工具会发送搜索词到所填站点，只读匿名内容，不使用桌面登录令牌。</small></span><input v-model="square" data-testid="mcp-square" type="checkbox" :disabled="busy" /></label>
    <label v-if="square" class="field"><span>广场站点</span><input v-model="base" data-testid="mcp-base" :disabled="busy" /></label>
    <div class="mcp-actions"><button class="button ghost-button" data-testid="mcp-generate" :disabled="busy || !executable.trim()" @click="generate">{{ busy ? '正在检查…' : '生成配置' }}</button><button v-if="config" class="button primary-button" :disabled="busy" @click="copy">复制配置</button></div>
    <p v-if="note" role="status">{{ note }}</p>
    <pre v-if="config" data-testid="mcp-config">{{ config }}</pre>
    <p>生成操作只检查路径，不运行所选程序，也不修改其他软件配置。将 JSON 添加到支持 mcpServers 的宿主；其他格式需填写相同 command/env，重新连接后生效。目录变化或程序移动后重新生成。</p>
  </section>
</template>
<script setup>
import { ref, watch, onUnmounted } from 'vue';
import { prepareMcpConfig } from '../platform/mcp.js';
import { copyLauncherText } from '../platform/paste.js';
const executable = ref(''), square = ref(false), base = ref('http://127.0.0.1:8787');
const config = ref(''), note = ref(''), busy = ref(false); let version = 0;
watch([executable,square,base], () => { ++version; config.value = ''; note.value = ''; });
onUnmounted(() => { ++version; });
async function generate() {
  if (busy.value) return;
  const current = ++version; busy.value = true; note.value = ''; config.value = '';
  try { const result = await prepareMcpConfig(executable.value,{square:square.value,base:base.value}); if (current===version) { config.value=result; note.value='路径已检查，请确认程序可信后配置宿主。'; } }
  catch (error) { if (current===version) note.value=error.message; }
  finally { busy.value=false; }
}
async function copy() {
  if (busy.value) return;
  busy.value = true;
  try { await copyLauncherText(config.value); note.value='配置已复制；尚未修改任何宿主设置。'; }
  catch { note.value='复制失败，可手动选择下方配置复制。'; }
  finally { busy.value=false; }
}
</script>
<style scoped>
.mcp-settings { margin-top: 32px; } h4 { font-size: 14px; font-weight: 600; } p { font-size: 12px; line-height: 1.7; color: var(--muted); } pre { padding: 16px; background: var(--sidebar); border: 1px solid var(--line); border-radius: 10px; white-space: pre-wrap; overflow-wrap: anywhere; font-size: 12px; user-select: text; } .mcp-actions { display: flex; gap: 8px; margin: 16px 0; }
</style>
