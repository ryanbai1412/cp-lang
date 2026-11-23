// Writes public/version.json, the commit this site was built from, so a
// reader (or an acceptance run) can tell which main the live site serves.
import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const commit = process.env.DEPLOY_SHA || execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
writeFileSync('public/version.json', `${JSON.stringify({ commit, built: new Date().toISOString() })}\n`);
console.log(`stamp: site built from ${commit.slice(0, 9)}`);
