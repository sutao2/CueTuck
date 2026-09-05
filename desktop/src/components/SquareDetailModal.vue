<template>
  <div class="modal-layer" data-testid="square-detail" @keydown.esc="$emit('cancel')">
    <div class="modal-backdrop" @click="$emit('cancel')"></div>
    <section class="modal create-modal" role="dialog" aria-modal="true" aria-labelledby="square-detail-title">
      <header class="modal-header">
        <div>
          <p class="modal-kicker">SQUARE</p>
          <h2 id="square-detail-title">{{ item.title }}</h2>
        </div>
        <button type="button" class="modal-close" aria-label="关闭" @click="$emit('cancel')">×</button>
      </header>
      <div class="create-body">
        <p v-if="loading" role="status">正在读取详情…</p>
        <div v-else-if="error" role="alert">
          <p>{{ error }}</p>
          <button type="button" class="button ghost-button" data-testid="square-detail-retry" @click="$emit('retry')">重试</button>
        </div>
        <template v-else>
          <p v-if="item.model" class="model-tag">{{ item.model }}</p>
          <pre class="square-body" data-testid="square-detail-content">{{ item.content || '还没有正文' }}</pre>
        </template>
        <p v-if="note" role="status">{{ note }}</p>
      </div>
      <footer class="modal-footer">
        <button type="button" class="button ghost-button" :disabled="favoriteBusy" @click="$emit('favorite')">{{ favorite ? '已收藏' : '收藏' }}</button>
        <button type="button" class="button primary-button" data-testid="square-detail-download" :disabled="loading || Boolean(error) || downloading" @click="$emit('download')">{{ downloading ? '正在下载…' : '下载到本地' }}</button>
      </footer>
    </section>
  </div>
</template>

<script setup>
defineProps({
  item: { type: Object, required: true },
  loading: Boolean,
  error: { type: String, default: '' },
  note: { type: String, default: '' },
  downloading: Boolean,
  favorite: Boolean,
  favoriteBusy: Boolean,
});
defineEmits(['cancel', 'retry', 'download', 'favorite']);
</script>

<style scoped>
.square-body { white-space: pre-wrap; overflow-wrap: anywhere; font: inherit; }
</style>
