import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

// The certificate is persistent; Tauri imports it into its temporary build keychain.
// Never generate a replacement identity during a build or log the certificate password.
export function previewEnvironment(env, platform, read = readFileSync) {
  const result = { ...env };
  if (platform !== 'darwin' || result.APPLE_SIGNING_IDENTITY) return result;
  const dir = resolve(root, 'output/private/macos-signing');
  try {
    const identity = JSON.parse(read(resolve(dir, 'identity.json'), 'utf8')).identity;
    if (!/^[A-Fa-f0-9]{40}$/.test(identity)) throw new Error('invalid identity');
    result.APPLE_SIGNING_IDENTITY = identity;
    result.APPLE_CERTIFICATE = read(resolve(dir, 'signing.p12')).toString('base64');
    result.APPLE_CERTIFICATE_PASSWORD = read(resolve(dir, 'password'), 'utf8').trim();
  } catch {
    throw new Error('Mac 预览包需要固定签名身份。请配置 APPLE_SIGNING_IDENTITY 或恢复本机签名备份；不能自动生成新身份或使用 ad-hoc 签名。');
  }
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const env = previewEnvironment(process.env, process.platform);
    const check = spawnSync(process.execPath, [resolve(root, 'scripts/release-check.mjs'), '--preview'], { env, stdio: 'inherit' });
    if (check.status !== 0) process.exit(check.status ?? 1);
    const build = spawnSync(process.execPath, [resolve(root, 'desktop/node_modules/@tauri-apps/cli/tauri.js'), 'build', '--bundles', 'app,dmg', '--config', 'src-tauri/tauri.preview.conf.json'], { cwd: resolve(root, 'desktop'), env, stdio: 'inherit' });
    process.exitCode = build.status ?? 1;
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
