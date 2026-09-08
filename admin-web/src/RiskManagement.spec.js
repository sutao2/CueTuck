import { mount, flushPromises } from '@vue/test-utils';
import { afterEach, expect, it, vi } from 'vitest';
import ReportManagement from './ReportManagement.vue';
import SafetyRules from './SafetyRules.vue';
import { resetAdminApi, setAdminApiTransport } from './adminApi.js';
let w; afterEach(()=>{w?.unmount();resetAdminApi();vi.restoreAllMocks();});
it('does not save default rules after a failed load',async()=>{setAdminApiTransport(()=>Promise.reject(Error('读取失败')));w=mount(SafetyRules);await flushPromises();expect(w.text()).toContain('读取失败');expect(w.find('[data-testid=rules-save]').exists()).toBe(false);expect(w.get('[data-testid=rule-add]').element.disabled).toBe(true);});
it('preserves rule drafts on conflict and tests saved rules without inventing success',async()=>{
 const calls=[];setAdminApiTransport(r=>{calls.push(r);if(r.kind==='rules')return {revision:0,rules:[]};if(r.kind==='rulesSave')throw Error('版本冲突');if(r.kind==='rulesTest')return {source:'local_rules',revision:0,score:0,hits:[],notice:'无命中不代表安全'};});
 w=mount(SafetyRules);await flushPromises();await w.get('[data-testid=rule-add]').trigger('click');await w.get('[aria-label=规则名称]').setValue('隐私');await w.get('[aria-label=关键词]').setValue('secret');await w.findAll('form')[0].trigger('submit');await flushPromises();expect(w.vm.hasUnsavedChanges).toBe(true);expect(w.text()).toContain('版本冲突');await w.get('[data-testid=rules-text]').setValue('sample');await w.findAll('form')[1].trigger('submit');await flushPromises();expect(w.text()).toContain('无命中不代表安全');expect(calls.find(r=>r.kind==='rulesTest').config).toEqual({text:'sample',category:null});
});
it('requires confirmation and retains report action on server conflict',async()=>{
 const report={id:'r1',target_id:'public',revision:2,status:'pending',priority:'normal',reason:'reported',events:[],rules:{hits:[]}};const write=vi.fn(()=>Promise.reject(Error('已结案')));
 setAdminApiTransport(r=>r.kind==='reports'?{items:[report],total:1,assignees:[]}:r.kind==='reportDetail'?report:write(r));
 w=mount(ReportManagement);await flushPromises();await w.get('[data-testid=report-open]').trigger('click');await flushPromises();await w.get('[data-testid=report-action]').setValue('offline');await w.get('[data-testid=report-reason]').setValue('verified');vi.spyOn(window,'confirm').mockReturnValue(false);await w.findAll('form')[1].trigger('submit');expect(write).not.toHaveBeenCalled();window.confirm.mockReturnValue(true);await w.findAll('form')[1].trigger('submit');await flushPromises();expect(write.mock.calls[0][0].config).toMatchObject({revision:2,action:'offline',reason:'verified'});expect(w.vm.hasUnsavedChanges).toBe(true);expect(w.text()).toContain('已结案');
});
