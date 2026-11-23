// What a mention of each repeated fact looks like, and what it must say.
const rules = (facts) => [
  { what: 'install.sh one-liner', find: /curl[^`"\n]*install\.sh[^`"\n#]*/g, ok: (s) => s.trim() === facts.install.sh },
  { what: 'install.ps1 one-liner', find: /&\s*\(\[scriptblock\][^`"\n#]*/g, ok: (s) => s.trim() === facts.install.ps1 },
  { what: 'version string', find: /\bcpl \d+\.\d+\.\d+[-+][\w<>&;.+-]*/g, ok: (s) => !/-dev\b/.test(s) && s.startsWith(`cpl ${facts.version}`) },
  { what: 'version string', find: /\b\d+\.\d+\.\d+-dev\+/g, ok: () => false },
  { what: 'saved-case folder (cpl saves to <file>.tests/)', find: /\b\w+\.cp\.tests\//g, ok: () => false },
];

/** Every line of `text` that states a fact differently from `facts`. */
export function contradictions(text, facts) {
  const out = [];
  text.split('\n').forEach((line, i) => {
    for (const r of rules(facts))
      for (const [m] of line.matchAll(r.find))
        if (!r.ok(m)) out.push({ line: i + 1, msg: `${r.what} \`${m.trim()}\` disagrees with web/lib/facts.json` });
  });
  return out;
}
