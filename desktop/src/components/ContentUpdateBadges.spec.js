import { mount, flushPromises } from '@vue/test-utils';
import { ref } from 'vue';
import { expect, it, vi } from 'vitest';
const state = vi.hoisted(() => ({ badges:null, refresh:vi.fn() }));
vi.mock('../platform/contentUpdateBadges.js', async importOriginal => ({
  ...await importOriginal(),
  useContentUpdateBadges: () => ({ badges:state.badges,refresh:state.refresh }),
}));
import WorkbenchShell from './WorkbenchShell.vue';
import { resetMemoryLibrary, createLocalPrompt, createLocalCollection } from '../platform/library.js';
import { resetMemorySession } from '../platform/session.js';
import ContentUpdates from './ContentUpdates.vue';
it('routes prompt and collection badges to the exact comparison without opening the card',async()=>{
 resetMemoryLibrary();resetMemorySession();
 const prompt=await createLocalPrompt({title:'Downloaded prompt',content:'My local edits'});
 const collection=await createLocalCollection({title:'Downloaded collection'});
 state.badges=ref({[`prompt:${prompt.id}`]:2,[`collection:${collection.id}`]:3});
 const w=mount(WorkbenchShell,{global:{stubs:{ContentUpdates:{props:['initialKey','badges'],template:'<div data-testid="updates"><button @click="$emit(\'close\')">Close</button></div>'}}}});
 try {
  await flushPromises();
  for(const [kind,item] of [['prompt',prompt],['collection',collection]]){
   const card=w.findAll('.prompt-card').find(card=>card.text().includes(item.title));
   await card.get('button.content-update-badge').trigger('click');await flushPromises();
   expect(w.findComponent(ContentUpdates).props('initialKey')).toBe(`${kind}:${item.id}`);
   expect(w.findComponent(ContentUpdates).props('badges')[`${kind}:${item.id}`]).toBe(kind==='prompt'?2:3);
   await w.get('[data-testid=updates] button').trigger('click');await flushPromises();
  }
 } finally {w.unmount();}
});
