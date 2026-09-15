<template><span><template v-for="(part,index) in parts" :key="index"><mark v-if="part.match">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template></span></template>
<script setup>
import { computed } from 'vue';
import { highlightedParts } from '../../../shared/search.js';
const props=defineProps({text:String,query:String});
const parts=computed(()=>highlightedParts(props.text,props.query));
</script>
<style scoped>mark{background:var(--search-highlight, #f0d97855);color:inherit;border-radius:2px;padding:0}</style>
