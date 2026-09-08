import { flushPromises, mount } from '@vue/test-utils';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import CatalogManagement from './CatalogManagement.vue';
import { resetAdminApi, setAdminApiTransport } from './adminApi.js';
let w;
const root = { id:'root', name:'图片生成', parent_id:null, enabled:true, sort_index:0, revision:2, icon:'folder', color:'#728080', vendor:'', group:'language', references:2 };
beforeEach(() => { resetAdminApi(); vi.spyOn(window,'confirm').mockReturnValue(true); });
afterEach(() => { w?.unmount(); vi.restoreAllMocks(); });
async function setup(write=vi.fn(), kind='categories', rows=[root]) {
  setAdminApiTransport(request => request.kind==='catalog' ? {items:rows} : write(request));
  w=mount(CatalogManagement,{props:{kind}}); await flushPromises();
}
it('creates a root or child and keeps the form on a failed save',async()=>{
  const write=vi.fn().mockRejectedValue(new Error('保存失败'));
  await setup(write); await w.get('[data-testid=catalog-new]').trigger('click');
  await w.get('[data-testid=catalog-name]').setValue('新分类');
  await w.get('[data-testid=catalog-parent]').setValue('root');
  await w.get('form').trigger('submit'); await flushPromises();
  expect(write.mock.calls[0][0].config).toMatchObject({name:'新分类',parent_id:'root'});
  expect(w.get('[data-testid=catalog-name]').element.value).toBe('新分类');
  expect(w.vm.hasUnsavedChanges).toBe(true); expect(w.text()).toContain('保存失败');
});
it('keeps IDs immutable and protects references and dirty drafts',async()=>{
  await setup(); await w.get('[data-testid=catalog-edit]').trigger('click');
  expect(w.find('[data-testid=catalog-id]').exists()).toBe(false);
  expect(w.get('[data-testid=catalog-delete]').element.disabled).toBe(true);
  await w.get('[data-testid=catalog-name]').setValue('草稿');
  window.confirm.mockReturnValue(false); await w.get('[data-testid=catalog-close]').trigger('click');
  expect(w.get('[data-testid=catalog-name]').element.value).toBe('草稿');
});
it('prevents duplicate submission and accepts only a confirmed revision',async()=>{
  let resolve; const write=vi.fn(()=>new Promise(done=>resolve=done));
  await setup(write,'models',[{...root,id:'Flux',name:'Flux',references:0}]);
  await w.get('[data-testid=catalog-edit]').trigger('click');
  await w.get('[data-testid=catalog-name]').setValue('FLUX');
  await w.get('form').trigger('submit'); await w.get('form').trigger('submit');
  expect(write).toHaveBeenCalledTimes(1); expect(w.vm.isBusy).toBe(true);
  resolve({}); await flushPromises(); expect(w.text()).toContain('服务端未确认'); expect(w.vm.hasUnsavedChanges).toBe(true);
});
it('requires confirmation for deletion and reports success only after the server confirms',async()=>{
  const write=vi.fn().mockResolvedValue({deleted:true}); await setup(write,'categories',[{...root,references:0}]);
  await w.get('[data-testid=catalog-edit]').trigger('click');
  window.confirm.mockReturnValue(false); await w.get('[data-testid=catalog-delete]').trigger('click'); expect(write).not.toHaveBeenCalled();
  window.confirm.mockReturnValue(true); await w.get('[data-testid=catalog-delete]').trigger('click'); await flushPromises();
  expect(write.mock.calls[0][0]).toMatchObject({kind:'catalogDelete',id:'root',config:{revision:2}}); expect(w.text()).toContain('已删除');
});
it('does not allow a failed list load to create or overwrite a dictionary',async()=>{
  setAdminApiTransport(()=>Promise.reject(new Error('加载失败'))); w=mount(CatalogManagement,{props:{kind:'categories'}}); await flushPromises();
  expect(w.get('[data-testid=catalog-new]').element.disabled).toBe(true); expect(w.text()).toContain('加载失败');
});
it('confirms migration with both revisions and preserves a blocked draft',async()=>{
  const write=vi.fn().mockResolvedValue({migrated:false,message:'先更换 Skill 分类'});await setup(write,'categories',[root,{...root,id:'target',name:'目标分类',revision:4}]);await w.findAll('[data-testid=catalog-edit]')[0].trigger('click');
  await w.get('[data-testid=catalog-migration-target]').setValue('target');await w.get('[data-testid=catalog-migration-reason]').setValue('合并');window.confirm.mockReturnValue(false);await w.get('.catalog-migration form').trigger('submit');expect(write).not.toHaveBeenCalled();
  window.confirm.mockReturnValue(true);await w.get('.catalog-migration form').trigger('submit');await flushPromises();expect(write.mock.calls[0][0].config).toMatchObject({target:'target',revision:2,target_revision:4,reason:'合并'});expect(w.vm.hasUnsavedChanges).toBe(true);expect(w.text()).toContain('先更换 Skill 分类');expect(w.get('[data-testid=catalog-migration-target]').element.value).toBe('target');
});
it('collapses categories but still reveals matching children when searching',async()=>{await setup(vi.fn(),'categories',[root,{...root,id:'child',parent_id:'root',name:'子分类'}]);expect(w.findAll('.catalog-list li')).toHaveLength(2);await w.findAll('.catalog-view-tools button')[0].trigger('click');expect(w.findAll('.catalog-list li')).toHaveLength(1);await w.get('input[type=search]').setValue('子分类');expect(w.findAll('.catalog-list li')).toHaveLength(1);expect(w.get('.catalog-list').text()).toContain('子分类')});
