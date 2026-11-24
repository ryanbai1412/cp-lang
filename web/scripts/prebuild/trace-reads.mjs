// Loaded with `node --import` into every prebuild stage (and the node
// processes it starts): records each file the stage reads and each path it
// hands to a child process, so prebuild can fail a stage whose declared
// inputs miss one (a cache keyed on them would go stale).
import childProcess from 'node:child_process';
import fs from 'node:fs';
import { syncBuiltinESMExports } from 'node:module';
import { resolve } from 'node:path';

const file = process.env.PREBUILD_TRACE;
if (file) {
  const seen = new Set();
  const note = (p) => {
    if (typeof p === 'string') seen.add(resolve(p));
    else if (p instanceof URL && p.protocol === 'file:') seen.add(resolve(decodeURIComponent(p.pathname)));
  };
  const wrap = (obj, names, pick = (args) => [args[0]]) => {
    for (const name of names) {
      const real = obj[name];
      if (typeof real !== 'function') continue;
      obj[name] = function (...args) {
        for (const p of pick(args)) note(p);
        return real.apply(this, args);
      };
    }
  };
  wrap(fs, ['readFileSync', 'readFile', 'readdirSync', 'readdir', 'opendirSync', 'opendir', 'createReadStream', 'copyFileSync', 'cpSync']);
  wrap(fs.promises, ['readFile', 'readdir', 'opendir', 'copyFile', 'cp']);
  const argPaths = (args) => [args[0], ...(Array.isArray(args[1]) ? args[1] : [])].filter((a) => typeof a === 'string' && fs.existsSync(a));
  wrap(childProcess, ['spawn', 'spawnSync', 'execFile', 'execFileSync'], argPaths);
  syncBuiltinESMExports();
  process.on('exit', () => fs.appendFileSync(file, [...seen].map((p) => `${p}\n`).join('')));
}
