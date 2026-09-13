import { readFileSync, writeFileSync, existsSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
const [directory, version] = process.argv.slice(2);
if (!directory || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version || '')) throw Error('Usage: node scripts/create-update-manifest.mjs <release-directory> <version>');
const root=resolve(directory), tag=`v${version}`;
const assets={ 'darwin-aarch64':'CueTuck.app.tar.gz', 'windows-x86_64':`CueTuck_${version}_x64-setup.exe` };
const platforms={};
for(const [platform,name] of Object.entries(assets)) {
 if(!existsSync(resolve(root,name))) throw Error(`Missing ${name}`);
 const signature=readFileSync(resolve(root,`${name}.sig`),'utf8').trim();
 if(!Buffer.from(signature,'base64').toString().startsWith('untrusted comment:')) throw Error(`Invalid signature format: ${name}`);
 platforms[platform]={signature,url:`https://github.com/sutao2/CueTuck/releases/download/${tag}/${name}`};
}
writeFileSync(resolve(root,'latest.json'),JSON.stringify({ version,notes:readFileSync(resolve(root,'RELEASE_NOTES.txt'),'utf8'),pub_date:new Date().toISOString(),platforms },null,2)+'\n');
const files=readdirSync(root).filter(name=>/\.(dmg|exe|gz|sig|json)$/.test(name)).sort();
writeFileSync(resolve(root,'SHA256SUMS'),files.map(name=>`${createHash('sha256').update(readFileSync(resolve(root,name))).digest('hex')}  ${name}\n`).join(''));
console.log(`Created signed-update manifest and hashes for ${Object.keys(platforms).join(', ')}`);
