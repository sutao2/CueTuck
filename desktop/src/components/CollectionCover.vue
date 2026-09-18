<template>
  <div v-if="type !== 'none' && urls.length" class="collection-cover" :class="[type === 'grid' ? 'collection-cover-grid' : 'collection-cover-single', `collection-cover-${variant}`]" :data-testid="type === 'grid' ? 'cover-grid' : 'cover-single'" aria-label="合集封面">
    <i v-for="(src, index) in cells" :key="index" :class="{ filled: Boolean(src) }">
      <img v-if="src" :src="src" alt="" loading="lazy" decoding="async">
    </i>
  </div>
</template>
<script setup>
import { computed } from 'vue';
import { parseCoverUrls } from '../lib/cover.js';
const props = defineProps({ type: { type: String, default: 'none' }, json: String, variant: { type: String, default: 'detail' } });
const urls = computed(() => parseCoverUrls(props.json));
const cells = computed(() => props.type === 'grid' ? Array.from({ length: 9 }, (_, i) => urls.value[i] || '') : urls.value.slice(0, 1));
</script>
<style scoped>
.collection-cover { display: grid; width: 100%; min-width: 0; overflow: hidden; border-radius: 10px; background: var(--sidebar, #f3f3f3); flex-shrink: 0; }
.collection-cover-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); grid-template-rows: repeat(3, minmax(0, 1fr)); aspect-ratio: 1; gap: 3px; }
.collection-cover i { display: block; min-width: 0; min-height: 0; overflow: hidden; background: var(--line, #e8e8e8); }
.collection-cover img { display: block; width: 100%; height: 100%; object-fit: cover; }
.collection-cover-detail { max-width: 480px; margin: 0 auto 24px; }
.collection-cover-single { aspect-ratio: 16 / 10; }
.collection-cover-single img { object-fit: contain; }
.collection-cover-card { margin: 0 0 14px; }
</style>
