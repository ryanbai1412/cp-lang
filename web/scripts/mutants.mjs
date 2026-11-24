// The one-token mutants of a program that practice-mutants.mjs runs on a
// problem's tests (site/R1, handbook/guidelines/site-checklist.md).
const operators = [
  [/<=/g, '<'], [/(?<![<\-])<(?![=<\-A-Z(])/g, '<='], [/>=/g, '>'], [/(?<![>\-=])>(?![=>])/g, '>='],
  [/==/g, '!='], [/!=/g, '=='], [/\.\.=/g, '..'], [/\.\.(?![=.])/g, '..='],
  [/ \+ 1\b/g, ''], [/ - 1\b/g, ''], [/ \+ (?!=)/g, ' - '], [/ - (?!=)/g, ' + '],
  [/\bmin\b/g, 'max'], [/\bmax\b/g, 'min'], [/\band\b/g, 'or'], [/\bor\b/g, 'and'],
  [/(?<![\w.])([0-9])(?![\w.])/g, (d) => String(Number(d) + 1)],
];

export function mutants(src) {
  const lines = src.split('\n');
  const out = [];
  lines.forEach((line, i) => {
    const code = line.replace(/#.*$|\/\/.*$/, '');
    if (/\bread\w*\s*[(<]/.test(code)) return;
    if (/^\s*(use|fn \w+\(.*\)\s*->|type|domain)\b/.test(code) && !/=/.test(code)) return;
    for (const [re, to] of operators) {
      for (const m of code.matchAll(new RegExp(re.source, 'g'))) {
        if (/"[^"]*$/.test(code.slice(0, m.index))) continue;
        const rep = typeof to === 'function' ? to(m[0]) : to;
        const next = line.slice(0, m.index) + rep + line.slice(m.index + m[0].length);
        out.push({ line: i + 1, from: line.trim(), to: next.trim(), src: [...lines.slice(0, i), next, ...lines.slice(i + 1)].join('\n') });
      }
    }
  });
  return out;
}
