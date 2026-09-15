import { expect,it } from 'vitest';
import { mount } from '@vue/test-utils';
import { searchTerms, matchesSearch } from '../../../shared/search.js';
import SearchHighlight from '../components/SearchHighlight.vue';
import { resetMemoryLibrary,createLocalPrompt,listLocalPrompts } from '../platform/library.js';
it('finds multilingual domain terms and corrects one English typo without broad short-word fuzziness',async()=>{
 expect(searchTerms('photograpy')).toContain('摄影');expect(searchTerms('imaeg')).toContain('图片');expect(searchTerms('图片')).toContain('image');expect(searchTerms('xxx')).toEqual(['xxx']);expect(matchesSearch('100abc','100%')).toBe(false);
 resetMemoryLibrary();await createLocalPrompt({title:'人物摄影',content:'portrait'});expect(await listLocalPrompts({query:'photograpy'})).toHaveLength(1);
});
it('highlights matched synonyms safely while retaining literal markup as text',()=>{
 const w=mount(SearchHighlight,{props:{text:'<img src=x onerror=alert(1)> 图片 image',query:'imaeg'}});
 expect(w.find('img').exists()).toBe(false);expect(w.findAll('mark').map(m=>m.text())).toEqual(['图片','image']);expect(w.text()).toContain('<img src=x');w.unmount();
});
