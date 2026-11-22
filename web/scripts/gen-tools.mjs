import { execFileSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const cpl = process.env.CPL ?? 'cpl';
const run = (...args) => execFileSync(cpl, args, { encoding: 'utf8' });

const help = run('--help');
const commands = [];
function splitUsage(rest) {
  const words = rest.trim().split(/\s+/);
  let depth = 0;
  let i = 0;
  for (; i < words.length; i++) {
    const w = words[i];
    if (depth === 0 && !/^[<[-]/.test(w) && !/^\.\.\./.test(w)) break;
    depth += (w.match(/[<[]/g) ?? []).length - (w.match(/[>\]]/g) ?? []).length;
  }
  return [words.slice(0, i).join(' '), words.slice(i).join(' ')];
}

let inCommands = false;
for (const line of help.split('\n')) {
  if (line.startsWith('commands:')) inCommands = true;
  else if (/^\S/.test(line)) inCommands = false;
  if (!inCommands) continue;
  const m = /^ {2}(\w+)(.*)$/.exec(line);
  if (m) {
    const [args, summary] = splitUsage(m[2]);
    commands.push({ name: m[1], usage: `${m[1]} ${args}`.trim(), summary });
  } else if (/^ {10,}\S/.test(line) && commands.length) {
    const c = commands.at(-1);
    c.summary = `${c.summary} ${line.trim()}`.trim();
  }
}
const missing = [];
for (const c of commands) {
  try {
    c.help = run(c.name, '--help').trimEnd();
  } catch {
    c.help = null;
    missing.push(c.name);
  }
}
if (missing.length) console.warn(`gen-tools: no --help for ${missing.join(', ')}`);

const judges = [];
for (const line of run('judges').split('\n')) {
  const head = /^([* ]) (\w+)\s+(.*)$/.exec(line);
  if (head) judges.push({ name: head[2], default: head[1] === '*', compiler: head[3], details: [] });
  else if (line.trim() && judges.length) judges.at(-1).details.push(line.trim());
}

const version = run('--version').trim();
writeFileSync('content/generated/cli.json', `${JSON.stringify({ version, commands, judges }, null, 2)}\n`);
console.log(`gen-tools: ${commands.length} commands, ${judges.length} judges (${version})`);
