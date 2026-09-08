import { mount,flushPromises } from '@vue/test-utils';
import { afterEach,expect,it } from 'vitest';
import ReportPanel from './ReportPanel.vue';
import { reportRequest,setReportTransport } from './reports.js';
import { resetMemorySession,setSessionTransport,loginSession } from './session.js';
let w;afterEach(()=>{w?.unmount();setReportTransport(null);resetMemorySession();});
it('requires login before a report API request',async()=>{await expect(reportRequest()).rejects.toThrow('需要登录');});
it('renders actual report progress and preserves failed submission',async()=>{setSessionTransport(async()=>({access_token:'report-token'}));await loginSession({email:'test@example.com',password:'pass'});setReportTransport(({config})=>{if(config)throw Error('发送失败');return {items:[{id:'r',target_id:'public',status:'closed',category:'other',reason:'原因',resolution:'已核查'}],total:1};});w=mount(ReportPanel,{props:{targetId:'public'}});await flushPromises();expect(w.text()).toContain('已下架结案');expect(w.text()).toContain('已核查');await w.get('textarea').setValue('举报草稿');await w.get('form').trigger('submit');await flushPromises();expect(w.get('textarea').element.value).toBe('举报草稿');expect(w.text()).toContain('发送失败');});
