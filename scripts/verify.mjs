import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, writeFile, open } from 'node:fs/promises';
import { dirname, resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const mode = process.argv[2] || 'all';
const node = process.execPath;
const npm = process.platform === 'win32' ? 'npm.cmd' : 'npm';
const steps = {
  frontend: [
    ['docs', '.', 'python3', ['scripts/docs-check']],
    ['api-contract', '.', node, ['scripts/api-contract-check.mjs']],
    ['tool-tests', '.', node, ['--test', 'scripts/release-check.test.mjs', 'scripts/import-community.test.mjs', 'scripts/api-contract-check.test.mjs']],
    ...['desktop', 'web', 'admin-web'].flatMap(dir => [[`${dir}-tests`, dir, npm, ['test']], [`${dir}-build`, dir, npm, ['run', 'build']]]),
  ],
  backend: [['backend', 'backend', 'cargo', ['test', '--locked', '--', '--test-threads=4']]],
  native: [['native', 'desktop/src-tauri', 'cargo', ['test', '--locked']]],
  mcp: [['mcp', 'mcp', 'cargo', ['test', '--locked']]],
  browser: [['browser', '.', node, ['scripts/browser-smoke.mjs']], ['admin-browser', '.', node, ['scripts/admin-browser-smoke.mjs']]],
};
if (mode !== 'all' && !steps[mode]) throw Error('Usage: node scripts/verify.mjs [all|frontend|backend|native|mcp|browser]');
const selected = mode === 'all' ? Object.values(steps).flat() : steps[mode];
const logRoot = resolve(root, 'output/verification');
await mkdir(logRoot, { recursive: true });
const logs = await mkdtemp(join(logRoot, 'run-'));
console.log(`Verification logs: ${logs}`);
const results = [];
for (const [name, cwd, command, args] of selected) {
  console.log(`Running ${name}…`);
  const log = await open(join(logs, `${name}.log`), 'w');
  const started = Date.now();
  const env = { ...process.env }; delete env.CARGO_TARGET_DIR;
  const code = await new Promise(resolveCode => {
    const child = spawn(command, args, { cwd: resolve(root, cwd), env, stdio: ['ignore', log.fd, log.fd] });
    child.on('error', error => { console.error(`${name}: ${error.message}`); resolveCode(1); });
    child.on('close', code => resolveCode(code ?? 1));
  });
  await log.close();
  results.push({ name, code, seconds: (Date.now() - started) / 1000 });
  console.log(`${code ? 'FAIL' : 'PASS'} ${name} (${results.at(-1).seconds.toFixed(1)}s)`);
}
await writeFile(join(logs, 'results.json'), JSON.stringify(results, null, 2));
process.exitCode = results.some(result => result.code !== 0) ? 1 : 0;
console.log(`${process.exitCode ? 'Verification failed' : 'Verification passed'}. Logs: ${logs}`);
