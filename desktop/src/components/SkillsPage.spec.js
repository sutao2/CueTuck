import {mount,flushPromises} from '@vue/test-utils';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import SkillsPage from './SkillsPage.vue';
import WorkbenchShell from './WorkbenchShell.vue';
import {setSkillsTransportForTests,groupSkills} from '../platform/skills.js';
import * as library from '../platform/library.js';
import {resetMemorySession} from '../platform/session.js';
import {resetSquare,setCatalogTransport,setSquareTransport} from '../platform/square.js';
let w,transport,snapshot,prepared;
const root=(id,scope='global',readonly=false)=>({id,name:id,scope,agent:'custom',path:`/isolated/${id}`,readonly,custom:true,shared_with:[],status:'目录已发现'});
const skill=(key,path,root_id='alpha')=>({key,path,physical_path:path,root_id,name:'同名 Skill',description:'实际描述',readonly:false,status:'外部安装',source:null,installed_digest:null,warnings:[]});
const packageData={name:'Example',description:'完整 Skill',body:'---\nname: Example\n---\n# Safe <script>bad()</script>',license:'未知',bytes:20,digest:'new',warnings:[],files:[{path:'SKILL.md',size:20,digest:'new',executable:false}]};
const button=(text,scope=w)=>scope.findAll('button').find(b=>b.text()===text);
async function click(text,scope=w){await button(text,scope).trigger('click');await flushPromises();}
async function mountPage(props={}){w=mount(SkillsPage,{props,attachTo:document.body});await flushPromises();}
beforeEach(()=>{
  snapshot={roots:[root('alpha'),root('beta'),root('project','project'),root('cache','global',true)],skills:[skill('one','/isolated/alpha/one'),skill('two','/isolated/beta/two','beta')],warnings:[],backups:[],operations:[],sources:[]};
  prepared={id:'preview',folder_name:'example',package:packageData,source:null,local_path:'/isolated/source'};
  transport=vi.fn(async request=>{
    switch(request.action){
      case 'snapshot':return structuredClone(snapshot);
      case 'detail':return {package:packageData,modified:false};
      case 'choose_directory':return '/isolated/source';
      case 'prepare_local':case 'prepare_remote':return structuredClone(prepared);
      case 'read_prepared':case 'read_file':return '# file body';
      case 'preflight':return request.root_ids.map(id=>({root_id:id,target:`/isolated/${id}/example`,status:'new',current_digest:null,changes:['+ SKILL.md'],message:'将创建独立副本'}));
      case 'install':return request.selections.map(s=>({target:`/isolated/${s.root_id}/example`,status:'success',message:'文件已就绪'}));
      case 'catalog':return {repo:'owner/repo',reference:'feature/skills',commit:'a'.repeat(40),license:'未知',entries:[{name:'example',directory:'skills/example'}]};
      default:return null;
    }
  });setSkillsTransportForTests(transport);
});
afterEach(()=>{w?.unmount();setSkillsTransportForTests(null);vi.restoreAllMocks();document.body.innerHTML='';});
it('does not merge names without a shared physical path or source identity',()=>{expect(groupSkills(snapshot.skills)).toHaveLength(2);expect(groupSkills([snapshot.skills[0],{...snapshot.skills[0],root_id:'beta'}])[0].installations).toHaveLength(2);});
it('shows a real desktop-only boundary in browsers without inventing skills',async()=>{setSkillsTransportForTests(null);await mountPage();expect(w.text()).toContain('浏览器无法扫描');expect(transport).not.toHaveBeenCalled();});
it('filters real local data, previews escaped source, and restores search on return',async()=>{await mountPage();expect(w.findAll('.skills-list-row')).toHaveLength(2);await w.get('[aria-label="搜索本机 Skills"]').setValue('不存在');expect(w.text()).toContain('没有匹配');await w.get('[aria-label="搜索本机 Skills"]').setValue('同名');await w.get('.skills-list-row').trigger('click');await flushPromises();expect(w.get('.skills-document').text()).toContain('<script>bad()</script>');expect(w.find('script').exists()).toBe(false);await w.get('.skills-back').trigger('click');await flushPromises();expect(w.get('[aria-label="搜索本机 Skills"]').element.value).toBe('同名');});
it('preflights explicit targets and does not install until confirmation',async()=>{await mountPage();await click('从文件夹安装');await click('安装到…');const dialog=w.get('[role="dialog"]');expect(dialog.findAll('.skills-target input').every(i=>!i.element.checked)).toBe(true);expect(dialog.findAll('.skills-target input').at(-1).attributes('disabled')).toBeDefined();await dialog.findAll('.skills-target input')[0].setValue(true);await click('比较并预览',dialog);expect(transport.mock.calls.some(([r])=>r.action==='install')).toBe(false);expect(dialog.text()).toContain('/isolated/alpha/example');await click('确认安装',dialog);expect(transport.mock.calls.find(([r])=>r.action==='install')[0].selections).toEqual([{root_id:'alpha',expected_digest:null,replace:false}]);expect(dialog.text()).toContain('安装完成');});
it('requires backup replacement acknowledgement for conflicts and dependency warnings',async()=>{
  prepared.package={...packageData,warnings:['外部依赖']};const base=transport.getMockImplementation();transport.mockImplementation(async r=>r.action==='preflight'?[{root_id:'alpha',target:'/isolated/alpha/example',status:'modified',current_digest:'local',changes:['~ SKILL.md'],message:'检测到本地修改'}]:base(r));
  await mountPage();await click('从文件夹安装');await click('安装到…');const dialog=w.get('[role="dialog"]');await dialog.get('.skills-target input').setValue(true);await click('比较并预览',dialog);expect(button('确认安装',dialog).attributes('disabled')).toBeDefined();for(const check of dialog.findAll('.skills-confirm-check input'))await check.setValue(true);expect(button('确认安装',dialog).attributes('disabled')).toBeUndefined();await click('确认安装',dialog);expect(transport.mock.calls.find(([r])=>r.action==='install')[0].selections[0]).toEqual({root_id:'alpha',expected_digest:'local',replace:true});
});
it('retries only failed positions after a partial installation',async()=>{
  const base=transport.getMockImplementation();transport.mockImplementation(async r=>r.action==='install'?[{target:'/isolated/alpha/example',status:'success',message:'ok'},{target:'/isolated/beta/example',status:'failed',message:'权限不足'}]:base(r));
  await mountPage();await click('从文件夹安装');await click('安装到…');const dialog=w.get('[role="dialog"]');await dialog.findAll('.skills-target input')[0].setValue(true);await dialog.findAll('.skills-target input')[1].setValue(true);await click('比较并预览',dialog);await click('确认安装',dialog);await click('仅重试失败位置',dialog);expect(transport.mock.calls.filter(([r])=>r.action==='preflight').at(-1)[0].root_ids).toEqual(['beta']);
});
it('lists pinned public catalog and reads the selected complete package',async()=>{await mountPage({mode:'square'});expect(w.text()).toContain('feature/skills');expect(w.text()).toContain('aaaaaaaaaaaa');await w.get('.skills-list-row').trigger('click');await flushPromises();const request=transport.mock.calls.find(([r])=>r.action==='prepare_remote')[0];expect(request.source.commit).toBe('a'.repeat(40));expect(request.source.directory).toBe('skills/example');});
it('shows failures without reporting an empty successful scan and retries the same action',async()=>{transport.mockRejectedValueOnce(new Error('目录不可读'));await mountPage();expect(w.get('[role="alert"]').text()).toContain('目录不可读');await click('重试');expect(w.findAll('.skills-list-row')).toHaveLength(2);});
it('retains persistent backups and asks before restoring',async()=>{snapshot.backups=[{id:'backup',path:'/isolated/backups/one',original:'/isolated/alpha/one',created_at:1,bytes:20,reason:'移除前备份',restored:false}];await mountPage();await click('备份与记录');await click('恢复到原位置');expect(transport.mock.calls.some(([r])=>r.action==='restore')).toBe(false);await click('恢复',w.get('[role="dialog"]'));expect(transport.mock.calls.some(([r])=>r.action==='restore'&&r.id==='backup')).toBe(true);});
it('keeps writes busy and prevents duplicate confirmation',async()=>{let finish;const base=transport.getMockImplementation();transport.mockImplementation(r=>r.action==='install'?new Promise(resolve=>finish=resolve):base(r));await mountPage();await click('从文件夹安装');await click('安装到…');const dialog=w.get('[role="dialog"]');await dialog.get('.skills-target input').setValue(true);await click('比较并预览',dialog);await button('确认安装',dialog).trigger('click');expect(button('取消',dialog).attributes('disabled')).toBeDefined();expect(w.emitted('busy').at(-1)).toEqual([true]);finish([{target:'/isolated/alpha/example',status:'success',message:'ok'}]);await flushPromises();});
it('preserves prompt drafts and filters when moving between sidebar groups',async()=>{
  library.resetMemoryLibrary();resetMemorySession();resetSquare();setCatalogTransport(async()=>({categories:[],models:[]}));setSquareTransport(async()=>[]);
  w=mount(WorkbenchShell,{attachTo:document.body});await flushPromises();await w.get('.inline-search input').setValue('retain');await w.get('.content-actions .primary-button').trigger('click');await w.get('[data-testid="prompt-editor"] input').setValue('draft');await w.get('[data-space="skills-local"]').trigger('click');expect(w.find('[data-testid="discard-editor"]').exists()).toBe(true);await click('继续编辑',w.get('[data-testid="prompt-editor"]'));expect(w.get('[data-testid="prompt-editor"] input').element.value).toBe('draft');await w.get('[data-space="skills-local"]').trigger('click');await click('放弃修改',w.get('[data-testid="prompt-editor"]'));await flushPromises();expect(w.get('[data-testid="skills-page"]').isVisible()).toBe(true);expect(w.get('.category-tree').isVisible()).toBe(false);await w.get('[data-space="local"]').trigger('click');await flushPromises();expect(w.get('.inline-search input').element.value).toBe('retain');
});
it('merges physical aliases and verified repository installs while preserving each version',()=>{const source={repo:'O/R',directory:'one',commit:'old'};const a={...skill('repo','/alpha'),source};const alias={...skill('alias','/link'),physical_path:'/alpha'};const newer={...skill('new','/beta'),source:{...source,repo:'o/r',commit:'new'}};const groups=groupSkills([a,alias,newer,skill('unrelated','/other')]);expect(groups).toHaveLength(2);expect(groups[0].installations).toHaveLength(3);expect(groups[0].installations[2].source.commit).toBe('new');});
it('blocks sidebar and settings shortcuts while the install confirmation is open',async()=>{
  library.resetMemoryLibrary();resetMemorySession();resetSquare();setCatalogTransport(async()=>({categories:[],models:[]}));setSquareTransport(async()=>[]);w=mount(WorkbenchShell,{attachTo:document.body,props:{host:'macos'}});await flushPromises();await w.get('[data-space="skills-local"]').trigger('click');await flushPromises();await click('从文件夹安装');await click('安装到…');await w.get('[data-space="local"]').trigger('click');window.dispatchEvent(new KeyboardEvent('keydown',{key:',',metaKey:true,bubbles:true}));await flushPromises();expect(w.get('[data-space="skills-local"]').attributes('aria-selected')).toBe('true');expect(w.get('[aria-labelledby="skills-install-title"]').exists()).toBe(true);expect(w.find('[data-testid="settings-view"]').exists()).toBe(false);
});
