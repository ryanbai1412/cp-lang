// What a mention of each repeated fact looks like, and what it must say.
const channels = (facts, shell) => [facts.install.edge[shell], facts.install.stable[shell]];
const rules = (facts) => [
  { what: 'install.sh one-liner', find: /curl[^`"\n]*install\.sh[^`"\n#]*/g, ok: (s) => channels(facts, 'sh').includes(s.trim()) },
  { what: 'install.ps1 one-liner', find: /&\s*\(\[scriptblock\][^`"\n#]*/g, ok: (s) => channels(facts, 'ps1').includes(s.trim()) },
  { what: 'version string', find: /\bcpl \d+\.\d+\.\d+[-+][\w<>&;.+-]*/g, ok: (s) => !/-dev\b/.test(s) && s.startsWith(`cpl ${facts.version}`) },
  { what: 'version string', find: /\b\d+\.\d+\.\d+-dev\+/g, ok: () => false },
  { what: 'saved-case folder (cpl saves to <file>.tests/)', find: /\b\w+\.cp\.tests\//g, ok: () => false },
  { what: 'retired domain method (0.3 spells them `.ord`, `.try_at`, `.count`)', find: /\.(to_idx|from_idx)\b/g, ok: () => false },
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
