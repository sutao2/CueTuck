<template>
  <section class="publication-preview" aria-label="发布预览" data-testid="publication-preview">
    <p class="preview-eyebrow">发布预览 · 审核通过后公开</p>
    <h2>{{ title }}</h2>
    <section v-if="cover"><h3>公开合集封面</h3><CollectionCover :type="cover.cover_type" :json="cover.cover_json" /></section>
    <p class="preview-meta">{{ category || '未分类' }} · {{ model || '通用模型' }}</p>
    <template v-if="members?.length"><article v-for="(member,index) in members" :key="index"><h3>{{ member.title }}</h3><pre>{{ member.content }}</pre></article></template>
    <pre v-else>{{ content || '没有正文' }}</pre>
    <h3>公开附件 <span>{{ assets.length }}</span></h3>
    <p v-if="!assets.length" class="preview-meta">{{ cover ? '仅公开上方合集封面，不公开成员附件。' : '本次不会公开任何附件或图片。' }}</p>
    <div v-else class="preview-files"><figure v-for="asset in assets" :key="asset.id"><img v-if="asset.mime.startsWith('image/')" :src="assetUrl(asset)" :alt="asset.name"><figcaption>{{ asset.name }}<small v-if="asset.memberTitle">{{ asset.memberTitle }}</small></figcaption></figure></div>
    <p class="preview-meta">确认正文和附件适合公开后提交。返回修改会保留当前选择。</p>
  </section>
</template>
<script setup>
import CollectionCover from './CollectionCover.vue';
import { assetUrl } from '../platform/assets.js';
defineProps({ cover:Object, title:String, content:String, category:String, model:String, members:Array, assets:{type:Array,default:()=>[]} });
</script>
<style scoped>
.publication-preview{box-sizing:border-box;flex:1;min-height:0;overflow:auto;padding:28px var(--page-gutter,32px);margin:0;width:100%;overflow-wrap:anywhere}.preview-eyebrow,.preview-meta,figcaption small{color:var(--text-secondary,#747474);font-size:13px}.publication-preview h2{font-size:24px;margin:12px 0}.publication-preview h3{font-size:15px;margin:24px 0 12px}.publication-preview pre{font:inherit;font-size:14px;line-height:1.8;white-space:pre-wrap;overflow-wrap:anywhere;margin:20px 0}.preview-files{display:grid;grid-template-columns:repeat(auto-fill,minmax(140px,1fr));gap:12px}.preview-files figure{margin:0;padding:12px;border:1px solid var(--border,#ddd);border-radius:10px}.preview-files img{width:100%;height:120px;object-fit:contain}.preview-files figcaption{font-size:12px}.preview-files small{display:block}
</style>
