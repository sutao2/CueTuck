import {expect,it} from 'vitest';
import {classifySkill,skillCategoryCounts} from './skillCategories.js';

it('classifies specific uses before falling back to a broad repository',()=>{
  expect(classifySkill({name:'frontend-design'},'vercel-labs/agent-skills')).toBe('design');
  expect(classifySkill({name:'api-design'})).toBe('development');
  expect(classifySkill({name:'system-design'})).toBe('development');
  expect(classifySkill({name:'artifact-template-system-design'})).toBe('office');
  expect(classifySkill({name:'code-review'},'getsentry/skills')).toBe('development');
  expect(classifySkill({name:'azure-active-directory-b2c'},'MicrosoftDocs/Agent-Skills')).toBe('cloud');
  expect(classifySkill({name:'modern-python'},'trailofbits/skills')).toBe('development');
  expect(classifySkill({name:'adaptyv'},'K-Dense-AI/claude-scientific-skills')).toBe('science');
  expect(classifySkill({name:'hf-cli'},'HUGGINGFACE/SKILLS')).toBe('ai');
});
it('keeps unknowns unclassified and ignores generic parent paths and trigger descriptions',()=>{
  expect(classifySkill({name:'my-skill',directory:'security/examples/my-skill',description:'Use for code, writing, marketing and AI'})).toBe('uncategorized');
  expect(classifySkill({name:'自定义工作流',path:'C:\\Users\\coding\\skills\\自定义工作流'})).toBe('uncategorized');
  expect(classifySkill({name:'数据库审计'})).toBe('security');
});
it('gives every item exactly one category and includes unknown items in the total',()=>{
  const items=[{name:'pdf'},{name:'data-analysis'},{name:'weather'},{name:'my-workflow'}].map(s=>({...s,category:classifySkill(s)}));
  const counts=skillCategoryCounts(items);
  expect(counts['']).toBe(4);expect(counts.office).toBe(1);expect(counts.science).toBe(1);expect(counts.tools).toBe(1);expect(counts.uncategorized).toBe(1);
  expect(Object.entries(counts).filter(([id])=>id).reduce((sum,[,n])=>sum+n,0)).toBe(counts['']);
});
