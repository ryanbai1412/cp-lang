// Writes generated/downloads.json from each release channel's published
// latest.json, so /setup links the editor plugins the release actually ships.
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';

const repo = 'https://github.com/ryanbai1412/cp-lang/releases/download';
const out = 'generated/downloads.json';
const channels = {};
try {
  for (const channel of ['stable', 'edge']) {
    const res = await fetch(`${repo}/${channel}/latest.json`);
    if (res.status === 404) continue;
    if (!res.ok) throw new Error(`${channel}/latest.json: HTTP ${res.status}`);
    const meta = await res.json();
    const file = (name) => meta.files?.[name] && { url: `${repo}/${channel}/${meta.files[name].file}`, size: meta.files[name].size };
    channels[channel] = { version: meta.version, date: meta.date, vscode: file('cp-lang.vsix'), jetbrains: file('cp-lang-jetbrains.zip') };
  }
} catch (e) {
  if (existsSync(out)) {
    console.log(`downloads: kept ${out} (${e.message})`);
    process.exit(0);
  }
  throw e;
}
if (!Object.keys(channels).length) throw new Error('downloads: no release channel has a latest.json');
mkdirSync('generated', { recursive: true });
writeFileSync(out, `${JSON.stringify(channels, null, 2)}\n`);
console.log(`downloads: ${Object.keys(channels).join(', ')}`);
