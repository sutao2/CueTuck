import { mount,flushPromises } from '@vue/test-utils';
import { afterEach,expect,it,vi } from 'vitest';
import ReportPanel from './ReportPanel.vue';
import { reportRequest,setReportTransport } from '../platform/reports.js';
import { resetMemorySession,setSessionTransport,loginSession } from '../platform/session.js';
let w;afterEach(()=>{w?.unmount();setReportTransport(null);resetMemorySession();vi.restoreAllMocks();});
async function login(){setSessionTransport(async()=>({access_token:'report-token'}));await loginSession({email:'test@example.com',password:'pass'});}
it('requires login and does not read local content',async()=>{await expect(reportRequest({target_id:'public'})).rejects.toThrow('需要登录');});
it('retains report reason after failure and shows only a confirmed submission',async()=>{await login();const write=vi.fn().mockRejectedValueOnce(Error('网络失败')).mockResolvedValueOnce({id:'report-1',status:'pending'});setReportTransport(({config})=>config?write(config):{items:[],total:0});w=mount(ReportPanel,{props:{targetId:'public'}});await w.get('textarea').setValue('具体原因');await w.get('form').trigger('submit');await flushPromises();expect(w.get('textarea').element.value).toBe('具体原因');expect(w.text()).toContain('网络失败');await w.get('form').trigger('submit');await flushPromises();expect(w.text()).toContain('举报已受理');expect(write.mock.lastCall[0]).toEqual({target_id:'public',category:'other',reason:'具体原因'});});
it('discards responses from a signed out session',async()=>{await login();let finish;setReportTransport(()=>new Promise(resolve=>finish=resolve));const pending=reportRequest();resetMemorySession();finish({items:[],total:0});await expect(pending).rejects.toThrow('会话已变化');});
