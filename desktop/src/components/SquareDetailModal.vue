<template>
    <section v-page-focus="() => $emit('cancel')" class="workspace-page square-reading-page" data-testid="square-detail" role="region" aria-labelledby="square-detail-title">
      <div class="detail-scroll">
      <nav class="detail-navigation" aria-label="详情导航"><button type="button" class="page-back" aria-label="返回" @click="$emit('cancel')">← 返回广场</button></nav>
      <header class="detail-heading">
        <div>
          <h2 id="square-detail-title">{{ item.title }}</h2>
          <div class="detail-meta"><span>{{ item.kind === 'collection' ? '提示词合集' : '提示词' }}</span><span v-if="item.model">{{ item.model }}</span><span v-if="item.author">{{ item.author }}</span></div>
        </div>
        <div class="detail-actions">
          <button v-if="downloaded && sourceImages.length && item.kind !== 'collection'" type="button" class="button ghost-button" :disabled="loading || Boolean(error) || downloading" data-testid="complete-square-images" @click="$emit('complete-images')">{{ downloading ? '正在补图…' : '补全参考图' }}</button>
          <button type="button" class="button ghost-button" :disabled="favoriteBusy" @click="$emit('favorite')">{{ favorite ? '已收藏' : '收藏' }}</button>
          <button type="button" class="button primary-button" data-testid="square-detail-download" :disabled="loading || Boolean(error) || downloading || (!downloaded && item.kind === 'collection' && !item.members?.length)" @click="$emit('download')">{{ downloading ? '正在下载…' : downloaded ? '打开本地副本' : '下载到本地' }}</button>
        </div>
      </header>
      <div class="detail-content">
        <p v-if="loading" role="status">正在读取详情…</p>
        <div v-else-if="error" role="alert">
          <p>{{ error }}</p>
          <button type="button" class="button ghost-button" data-testid="square-detail-retry" @click="$emit('retry')">重试</button>
        </div>
        <template v-else>
          <section v-if="sourceImages.length" class="detail-gallery" aria-label="来源参考图">
            <div class="gallery-label"><span>来源参考图 · 非本软件生成</span></div>
            <figure>
              <div class="gallery-stage">
                <button v-if="!imageFailed" type="button" class="gallery-open" aria-label="查看大图" @click="largeImage = activeImage"><img :key="imageKey" :src="activeImage.url" :alt="activeImage.alt" referrerpolicy="no-referrer" @error="imageFailed = true" @load="imageLoaded = true"></button>
                <div v-if="imageFailed" class="gallery-fallback" role="status">图片暂时无法加载 <button class="button" type="button" @click="retryImage">重试图片</button></div>
                <span v-else-if="!imageLoaded" class="gallery-loading" role="status">正在加载图片…</span>
              </div>
              <figcaption><span>{{ activeImage.alt }}</span><a v-if="activeImage.source" :href="activeImage.source" target="_blank" rel="noopener noreferrer">{{ item.reference.author }} ↗</a></figcaption>
            </figure>
            <div class="gallery-thumbs" aria-label="选择预览图片">
              <button v-for="(image, index) in galleryImages" :key="image.url" type="button" :aria-label="`预览图片 ${index + 1}`" :aria-pressed="imageIndex === index" @click="selectImage(index)"><img :src="image.url" alt="" loading="lazy" referrerpolicy="no-referrer"><span>{{ index + 1 }}</span></button>
            </div>
          </section>
          <p v-if="item.reference" class="reference-credit"><a v-if="referenceLink(item.reference.url)" :href="referenceLink(item.reference.url)" target="_blank" rel="noopener noreferrer">{{ item.reference.repository }} ↗</a> · {{ item.reference.author }} · <a v-if="referenceLink(item.reference.license_url)" :href="referenceLink(item.reference.license_url)" target="_blank" rel="noopener noreferrer">{{ item.reference.license }}</a></p>
          <h3 class="detail-section-label">{{ item.kind === 'collection' ? '合集内容' : '提示词正文' }}</h3>
          <template v-if="item.kind === 'collection'">
            <p>{{ item.members?.length || 0 }} 个提示词</p>
            <p v-if="!item.members?.length">该合集缺少成员快照，暂时无法下载。</p>
            <article v-for="(member, index) in item.members" :key="index" data-testid="square-detail-member">
              <h3>{{ member.title }}</h3>
              <p v-if="member.model">{{ member.model }}</p>
              <pre class="square-body">{{ member.content }}</pre>
              <PublishedAttachments :item-id="item.id" :references="(item.asset_refs || []).filter(file => member.asset_ids?.includes(file.id))" />
            </article>
          </template>
          <pre v-else class="square-body" data-testid="square-detail-content">{{ item.content || '还没有正文' }}</pre>
          <PublishedAttachments v-if="item.kind !== 'collection'" :item-id="item.id" :references="item.asset_refs || []" />
        </template>
        <p v-if="note" role="status">{{ note }}</p>
        <ReportPanel v-if="!loading && !error" :key="item.id" :target-id="item.id" />
      </div>
      </div>
      <ImageViewer v-if="largeImage" :src="largeImage.url" :title="largeImage.alt" @close="largeImage = null" />
    </section>
</template>

<script setup>
import { vPageFocus } from "../lib/pageFocus.js";
import ReportPanel from './ReportPanel.vue';
import PublishedAttachments from './PublishedAttachments.vue';
import ImageViewer from './ImageViewer.vue';
import { computed, ref, watch } from 'vue';
import { referenceImages, referenceLink } from '../lib/squareReference.js';
const props = defineProps({
  item: { type: Object, required: true },
  loading: Boolean,
  error: { type: String, default: '' },
  note: { type: String, default: '' },
  downloading: Boolean,
  downloaded: Boolean,
  favorite: Boolean,
  favoriteBusy: Boolean,
});
defineEmits(['cancel', 'retry', 'download', 'favorite', 'complete-images']);
const largeImage = ref(null);
const imageIndex = ref(0), imageFailed = ref(false), imageLoaded = ref(false), imageKey = ref(0);
const sourceImages = computed(() => referenceImages(props.item));
const galleryImages = computed(() => sourceImages.value.map(url => ({ url, alt: props.item.title, source: referenceLink(props.item.reference.url) })));
const activeImage = computed(() => galleryImages.value[imageIndex.value] || galleryImages.value[0]);
function retryImage() { imageFailed.value = false; imageLoaded.value = false; imageKey.value++; }
function selectImage(index) { imageIndex.value = index; retryImage(); }
watch(() => props.item.id, () => { largeImage.value=null; selectImage(0); });
</script>

<style scoped>
.square-body { white-space: pre-wrap; overflow-wrap: anywhere; font: inherit; }
.square-reading-page { overflow: hidden; }
.detail-scroll { overflow-y: auto; padding: 24px max(28px, calc((100% - 880px) / 2)) 48px; min-height: 0; }
.detail-navigation { margin-bottom: 22px; }
.detail-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 20px; margin-bottom: 24px; }
.detail-heading > div { min-width: 0; }
.detail-heading h2 { margin: 0; font-size: 26px; font-weight: 600; line-height: 1.4; overflow-wrap: anywhere; }
.detail-meta { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; color: var(--muted); font-size: 12px; }
.detail-meta span + span::before { content: '·'; margin-right: 8px; }
.detail-actions { display: flex; flex-shrink: 0; gap: 8px; padding-top: 3px; }
.detail-section-label { font-size: 12px; font-weight: 600; color: var(--muted); margin: 28px 0 12px; }
.detail-content .square-body { margin: 0; padding: 0; border: 0; background: transparent; font-size: 14px; line-height: 1.9; }
.reference-credit { color: var(--muted); font-size: 11px; line-height: 1.7; overflow-wrap: anywhere; }
.reference-credit a { color: inherit; }
.gallery-label { display: flex; justify-content: space-between; gap: 12px; color: var(--muted); font-size: 11px; margin-bottom: 10px; }
.gallery-label button { border: 0; background: transparent; color: inherit; }
.detail-gallery figure { margin: 0; }
.gallery-stage { position: relative; aspect-ratio: 16 / 9; background: var(--sidebar); border: 1px solid var(--line); border-radius: 12px; overflow: hidden; display: grid; place-items: center; }
.gallery-open { width: 100%; height: 100%; padding: 0; border: 0; background: transparent; min-height: 0; cursor: zoom-in; }
.gallery-open > img { width: 100%; height: 100%; object-fit: contain; min-height: 0; }
.gallery-loading { position: absolute; color: var(--muted); font-size: 12px; }
.gallery-fallback { display: grid; justify-items: center; gap: 12px; color: var(--muted); }
figcaption { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 8px; color: var(--muted); font-size: 11px; margin-top: 10px; }
figcaption a { color: inherit; text-decoration: none; }
figcaption a:hover { text-decoration: underline; }
.gallery-thumbs { display: flex; gap: 8px; margin-top: 14px; }
.gallery-thumbs button { position: relative; width: 76px; height: 50px; padding: 3px; border: 1px solid var(--line); border-radius: 7px; background: var(--surface); overflow: hidden; }
.gallery-thumbs button[aria-pressed='true'] { border-color: var(--text); }
.gallery-thumbs img { width: 100%; height: 100%; object-fit: cover; border-radius: 3px; }
.gallery-thumbs span { position: absolute; bottom: 3px; right: 4px; color: white; background: #0009; padding: 0 4px; font-size: 10px; border-radius: 3px; }
.detail-content :deep(.report-panel) { margin-top: 32px; padding-top: 16px; border-color: var(--line); --border-color: var(--line); --panel-bg: var(--surface); }
.detail-content :deep(.report-panel > summary) { color: var(--muted); font-size: 11px; }
@media (max-width: 1000px) { .detail-heading { flex-direction: column; gap: 14px; } .detail-heading h2 { font-size: 23px; } }
</style>
