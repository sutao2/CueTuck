import {mount,flushPromises} from '@vue/test-utils';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
const service=vi.hoisted(()=>({contentVersions:vi.fn(),localContentUpdate:vi.fn(),prepareVersionDownload:vi.fn()}));
vi.mock('../platform/contentVersions.js',()=>service);
import ContentUpdates from './ContentUpdates.vue';
let wrapper,known;
const row={local_id:'local',kind:'prompt',remote_id:'public',version:1,title:'My copy'};
beforeEach(()=>{known=true;service.prepareVersionDownload.mockResolvedValue({prompts:[{title:'New',content:'v2'}]});service.contentVersions.mockImplementation(async(kind,id,version)=>version?{id,kind,version,snapshot:{title:'New',content:'v2'},change_note:'Improved'}:{current_version:2});service.localContentUpdate.mockImplementation(async request=>request.action==='list'?[{...row,version:known?1:null}]:request.action==='preview'?{local_digest:'checked',modified:known,local:{root:{title:'My copy',content:'user edit'}},origin:known?{version:1,remote:{title:'Old',content:'v1'}}:null}:{});});
afterEach(()=>{wrapper?.unmount();vi.clearAllMocks();});
async function click(text){await wrapper.findAll('button').find(b=>b.text()===text).trigger('click');await flushPromises();}
it('requires explicit replacement confirmation and passes the preview digest',async()=>{wrapper=mount(ContentUpdates);await flushPromises();await click('检查更新');expect(wrapper.text()).toContain('本地存在修改');await click('备份后替换');expect(service.localContentUpdate.mock.calls.some(([r])=>r.action==='apply')).toBe(false);await click('确认备份并替换');expect(service.localContentUpdate).toHaveBeenCalledWith(expect.objectContaining({action:'apply',expected_digest:'checked',replace:true,version:2,remote_id:'public'}));expect(wrapper.emitted('changed')).toHaveLength(1);});
it('allows only a new copy when legacy download baseline is unknown',async()=>{known=false;wrapper=mount(ContentUpdates);await flushPromises();await click('检查更新');expect(wrapper.findAll('button').find(b=>b.text()==='备份后替换').element.disabled).toBe(true);await click('保留本地并另存新版');expect(service.localContentUpdate).toHaveBeenCalledWith(expect.objectContaining({action:'apply',replace:false,expected_digest:'checked'}));});
it('keeps the preview visible and reports a concurrent-edit conflict without reporting success',async()=>{const original=service.localContentUpdate.getMockImplementation();service.localContentUpdate.mockImplementation(r=>r.action==='apply'?Promise.reject(Error('预览后本地内容已变化')):original(r));wrapper=mount(ContentUpdates);await flushPromises();await click('检查更新');await click('备份后替换');await click('确认备份并替换');expect(wrapper.get('[role=alert]').text()).toContain('已变化');expect(wrapper.emitted('changed')).toBeUndefined();expect(wrapper.text()).toContain('user edit');});
it('opens the exact badged collection for comparison without downloading or applying',async()=>{
 const original=service.localContentUpdate.getMockImplementation();
 service.localContentUpdate.mockImplementation(r=>r.action==='list'?[row,{...row,kind:'collection',title:'Collection'}]:original(r));
 wrapper=mount(ContentUpdates,{props:{initialKey:'collection:local',badges:{'collection:local':2}}});await flushPromises();
 expect(service.localContentUpdate).toHaveBeenCalledWith({action:'preview',kind:'collection',local_id:'local'});
 expect(wrapper.text()).toContain('Collection');expect(wrapper.text()).toContain('user edit');
 expect(service.prepareVersionDownload).not.toHaveBeenCalled();
 expect(service.localContentUpdate.mock.calls.every(([r])=>['list','preview'].includes(r.action))).toBe(true);
});
