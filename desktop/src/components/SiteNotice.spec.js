import {mount} from '@vue/test-utils';import {it,expect,vi,afterEach} from 'vitest';import SiteNotice from '../../../shared/SiteNotice.vue';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
let w;afterEach(()=>{w?.unmount();vi.useRealTimers();});
it('renders announcements as text and expires them without refreshing',async()=>{vi.useFakeTimers();vi.setSystemTime(new Date('2026-09-08T00:00:00Z'));w=mount(SiteNotice,{props:{site:{name:'社区',description:'说明',announcement:'<script>alert(1)</script>',announcement_end:'2026-09-08T00:00:02Z',publishing_open:false}}});expect(w.find('script').exists()).toBe(false);expect(w.text()).toContain('<script>');expect(w.text()).toContain('关闭新投稿');await vi.advanceTimersByTimeAsync(2000);expect(w.text()).not.toContain('<script>');});
it('keeps offline empty configuration unobtrusive and rejects executable image URLs',async()=>{w=mount(SiteNotice);expect(w.find('section').exists()).toBe(false);await w.setProps({site:{name:'社区',logo_url:'javascript:alert(1)'}});expect(w.find('img').exists()).toBe(false);await w.setProps({site:{name:'社区',logo_url:'https://example.com/logo.png'}});expect(w.get('img').attributes('referrerpolicy')).toBe('no-referrer');await w.get('img').trigger('error');expect(w.find('img').exists()).toBe(false);});
it('permits HTTPS site images in the native package without opening script or connection policy',()=>{
  const config=JSON.parse(readFileSync(resolve(process.cwd(),'src-tauri/tauri.conf.json'),'utf8'));
  const directives=Object.fromEntries(config.app.security.csp.split(';').map(s=>s.trim().split(/\s+/)).map(([key,...value])=>[key,value]));
  expect(directives['img-src']).toContain('https:');expect(directives['script-src']).toEqual(["'self'"]);expect(directives['connect-src']).not.toContain('https:');
});
