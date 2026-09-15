<template>
  <section class="content-state" :class="[kind, { compact }]" :role="kind==='error'?'alert':'status'" :aria-busy="kind==='loading'">
    <AppIcon :name="kind==='loading'?'refresh':kind==='error'?'globe':icon" class="state-icon" />
    <div class="state-copy"><h3>{{ title }}</h3><p v-if="description">{{ description }}</p></div>
    <div v-if="$slots.default" class="state-actions"><slot /></div>
    <div v-if="kind==='loading'&&!compact" class="state-skeleton" aria-hidden="true"><i/><i/><i/></div>
  </section>
</template>
<script setup>
import AppIcon from './AppIcon.vue';
defineProps({kind:{type:String,default:'empty'},title:String,description:String,icon:{type:String,default:'search'},compact:Boolean});
</script>
<style scoped>
.content-state{box-sizing:border-box;display:flex;flex-direction:column;align-items:center;gap:14px;text-align:center;padding:36px 24px;margin:16px 0;border:1px solid var(--border,#ddd);border-radius:12px;min-height:0}.state-icon{width:26px;height:26px;opacity:.6}.state-copy{min-width:0;overflow-wrap:anywhere}.state-copy h3{font-size:16px;line-height:1.5;margin:0}.state-copy p{font-size:13px;line-height:1.6;margin:6px 0 0;color:var(--text-secondary,#747474)}.state-actions{display:flex;justify-content:center;flex-wrap:wrap;gap:8px}.state-skeleton{width:min(100%,640px);display:grid;gap:10px}.state-skeleton i{height:12px;border-radius:4px;background:var(--border,#ddd);opacity:.5}.state-skeleton i:last-child{width:65%}.compact{flex-direction:row;text-align:left;padding:16px 20px;gap:12px}.compact .state-copy{flex:1}.error{border-color:#b46a6a77}@media(max-width:600px){.compact{flex-wrap:wrap}.compact .state-actions{width:100%;justify-content:flex-start}}
</style>
