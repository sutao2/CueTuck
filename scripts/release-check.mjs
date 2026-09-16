import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { normalizeApiBase } from '../shared/apiBase.js';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = path => readFileSync(resolve(root, path), 'utf8');

export function validateRelease({ config, version, cargoVersion, frontendUpdates, nativeUpdates, env = {}, production = false, preview = false, platform = process.platform }) {
  const errors = [];
  if (config.version !== version || cargoVersion !== version) errors.push('Desktop package/Cargo/Tauri versions differ');
  const endpoint = config.plugins?.updater?.endpoints?.[0] || '';
  const repo = endpoint.match(/^https:\/\/github.com\/([^/]+\/[^/]+)\/releases\/latest\/download\/latest\.json$/)?.[1];
  if (!repo || !frontendUpdates.includes(`https://api.github.com/repos/${repo}/releases`) || !nativeUpdates.includes(`https://api.github.com/repos/${repo}/releases`) || !nativeUpdates.includes(`https://github.com/${repo}/releases/download/`)) errors.push('Updater repository/endpoints do not agree');
  const key = Buffer.from(config.plugins?.updater?.pubkey || '', 'base64').toString();
  if (!key.startsWith('untrusted comment:') || !/^RW[A-Za-z0-9+/=]+$/m.test(key)) errors.push('Updater public key is missing or invalid');
  if (preview && (!/^\d+\.\d+\.\d+-[\w.-]+$/.test(version) || config.bundle?.createUpdaterArtifacts !== true)) errors.push('Preview requires a prerelease version and signed updater artifacts');
  if (preview && platform === 'darwin' && (!env.APPLE_SIGNING_IDENTITY?.trim() || env.APPLE_SIGNING_IDENTITY.trim() === '-')) errors.push('Mac preview requires a persistent APPLE_SIGNING_IDENTITY; ad-hoc signing loses Keychain authorization across updates');
  if (production || preview) {
    if (!env.PROMPTARK_API_BASE || !env.VITE_API_BASE) errors.push('Release requires PROMPTARK_API_BASE and VITE_API_BASE');
    else try {
      const native = normalizeApiBase(env.PROMPTARK_API_BASE), web = normalizeApiBase(env.VITE_API_BASE);
      if (native !== web || !native.startsWith('https://') || ['localhost', '127.0.0.1', '[::1]'].includes(new URL(native).hostname)) errors.push('Release API origins must match and use non-loopback HTTPS');
    } catch { errors.push('Invalid release API origin'); }
    if ((production || preview) && !env.TAURI_SIGNING_PRIVATE_KEY) errors.push('Release requires the matching updater signing key (not generated automatically)');
  }
  return errors;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const production = process.argv.includes('--production');
  const preview = process.argv.includes('--preview');
  const config = JSON.parse(read('desktop/src-tauri/tauri.conf.json'));
  if (preview) config.bundle = { ...config.bundle, ...JSON.parse(read('desktop/src-tauri/tauri.preview.conf.json')).bundle };
  const errors = validateRelease({ config, version: JSON.parse(read('desktop/package.json')).version, cargoVersion: read('desktop/src-tauri/Cargo.toml').match(/^version = "([^"]+)"/m)?.[1], frontendUpdates: read('desktop/src/platform/updates.js'), nativeUpdates: read('desktop/src-tauri/src/commands/updates.rs'), env: process.env, production, preview });
  errors.forEach(error => console.error(error));
  process.exitCode = errors.length ? 1 : 0;
  if (!errors.length) console.log(preview ? 'Preview checks passed. Signed updater artifacts required; Apple signing/notarization is not verified.' : `${production ? 'Release configuration' : 'Local build configuration'} checks passed. Signing-key match, notarization and hosted update artifacts still require release verification.`);
}
