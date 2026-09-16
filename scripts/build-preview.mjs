import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

// The certificate and its dedicated build keychain are persistent.
// Never generate a replacement identity during a build or log the certificate password.
export function previewEnvironment(env, platform, read = readFileSync) {
  const result = { ...env };
  if (platform !== 'darwin' || result.APPLE_SIGNING_IDENTITY) return result;
  const dir = resolve(root, 'output/private/macos-signing');
  try {
    const { identity, keychain } = JSON.parse(read(resolve(dir, 'identity.json'), 'utf8'));
    if (typeof keychain !== 'string' || !keychain.endsWith('.keychain-db')) throw new Error('invalid keychain');
    if (!/^[A-Fa-f0-9]{40}$/.test(identity)) throw new Error('invalid identity');
    result.APPLE_SIGNING_IDENTITY = identity;
    result.CUETUCK_SIGNING_KEYCHAIN = keychain;
  } catch {
    throw new Error('Mac 预览包需要固定签名身份。请配置 APPLE_SIGNING_IDENTITY 或恢复本机签名备份；不能自动生成新身份或使用 ad-hoc 签名。');
  }
  return result;
}

function security(args) {
  const result = spawnSync('/usr/bin/security', args, { encoding: 'utf8' });
  if (result.status !== 0) throw new Error('无法准备专用构建钥匙串，请检查本机签名材料');
  return result.stdout;
}
function keychains() { return security(['list-keychains', '-d', 'user']).trim().split('\n').filter(Boolean).map(line => JSON.parse(line.trim())); }
function prepareKeychain(env) {
  const path = env.CUETUCK_SIGNING_KEYCHAIN;
  if (!path) return () => {};
  const password = readFileSync(resolve(root, 'output/private/macos-signing/password'), 'utf8').trim();
  security(['unlock-keychain', '-p', password, path]);
  const original = keychains();
  if (original.includes(path)) return () => {};
  security(['list-keychains', '-d', 'user', '-s', ...original, path]);
  return () => security(['list-keychains', '-d', 'user', '-s', ...keychains().filter(item => item !== path)]);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const env = previewEnvironment(process.env, process.platform);
    const check = spawnSync(process.execPath, [resolve(root, 'scripts/release-check.mjs'), '--preview'], { env, stdio: 'inherit' });
    if (check.status !== 0) process.exit(check.status ?? 1);
    const cleanup = prepareKeychain(env);
    try {
      const build = spawnSync(process.execPath, [resolve(root, 'desktop/node_modules/@tauri-apps/cli/tauri.js'), 'build', '--bundles', 'app,dmg', '--config', 'src-tauri/tauri.preview.conf.json'], { cwd: resolve(root, 'desktop'), env, stdio: 'inherit' });
      process.exitCode = build.status ?? 1;
    } finally { cleanup(); }
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
