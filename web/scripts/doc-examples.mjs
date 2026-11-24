// Presentation helpers for the reference pages gen-ref.mjs writes. What a
// doc says (facts, examples, prose) comes parsed from `cpl doc`.

// Splits `s` at top-level occurrences of `sep`, outside brackets and strings.
export function splitTop(s, sep) {
  const parts = [];
  let depth = 0;
  let quote = null;
  let start = 0;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (quote) {
      if (c === '\\') i++;
      else if (c === quote) quote = null;
      continue;
    }
    if (c === '"' || c === "'") quote = c;
    else if ('([{'.includes(c)) depth++;
    else if (')]}'.includes(c)) depth--;
    else if (depth === 0 && s.startsWith(sep, i)) {
      parts.push(s.slice(start, i));
      start = i + sep.length;
      i += sep.length - 1;
    }
  }
  parts.push(s.slice(start));
  return parts.map((p) => p.trim()).filter((p, i, a) => p || i < a.length - 1);
}

// An example that reads input states it after the code: (stdin `4 5`), or
// (empty stdin); `stdin` is null for one that states none.
// The leading statements a type's examples share: of the prefixes that at
// least two examples start with, the one that hides the most lines.
export function commonSetup(examples) {
  let best = [];
  let saved = 0;
  for (const e of examples)
    for (let k = 1; k <= e.setup.length; k++) {
      const head = e.setup.slice(0, k);
      const n = examples.filter((x) => head.every((st, i) => x.setup[i] === st)).length;
      if (n >= 2 && n * k > saved) {
        best = head;
        saved = n * k;
      }
    }
  return best;
}

// `fn f<T>( self, a: A, b: B = x, ) -> R` from `cpl doc`, laid out like
// `cpl fmt`: on one line when it fits in `width`, else one parameter per line.
export function layoutSignature(sig, width = 72) {
  const open = sig.indexOf('(');
  if (open < 0) return sig;
  let depth = 0;
  let close = -1;
  for (let i = open; i < sig.length; i++) {
    if ('([{<'.includes(sig[i]) && !(sig[i] === '<' && sig[i - 1] === ' ')) depth++;
    else if (')]}'.includes(sig[i]) || (sig[i] === '>' && sig[i - 1] !== '-' && sig[i - 1] !== '=' && depth > 1)) depth--;
    if (depth === 0 && sig[i] === ')') {
      close = i;
      break;
    }
  }
  if (close < 0) return sig;
  const head = sig.slice(0, open);
  const params = splitTop(sig.slice(open + 1, close), ',').filter(Boolean);
  const tail = sig.slice(close + 1).trim();
  const one = `${head}(${params.join(', ')})${tail ? ' ' + tail : ''}`;
  if (one.length <= width || params.length < 2) return one;
  return `${head}(\n${params.map((p) => `    ${p},`).join('\n')}\n)${tail ? ' ' + tail : ''}`;
}

// Math in docs is LaTeX in `\( ... \)`; the site renders it with KaTeX at
// build time as `$$...$$` (remark-math, single dollars off). Code spans and
// fences are left alone.
const outsideCode = (text, f) =>
  text
    .split(/(```[\s\S]*?```|`[^`\n]*`)/)
    .map((part, i) => (i % 2 ? part : f(part)))
    .join('');

// A cost written before the convention, `O(n log n)`, as LaTeX.
function legacyTex(inner) {
  return inner
    .replace(/\bsqrt\(([^()]*)\)/g, '\\sqrt{$1}')
    .replace(/√\(([^()]*)\)/g, '\\sqrt{$1}')
    .replace(/√(\w+)/g, '\\sqrt{$1}')
    .replace(/\blog\b/g, '\\log')
    .replace(/[·*]/g, '\\cdot ')
    .replace(/α/g, '\\alpha')
    .replace(/²/g, '^2')
    .replace(/³/g, '^3')
    .replace(/\^(\w{2,}|\([^)]*\))/g, (_, e) => `^{${e.replace(/^\(|\)$/g, '')}}`)
    .replace(/(\\log|\\alpha|\\sqrt)\s+/g, '$1 ');
}

function balancedEnd(s, open) {
  let depth = 0;
  for (let i = open; i < s.length; i++) {
    if (s[i] === '(') depth++;
    else if (s[i] === ')' && --depth === 0) return i;
  }
  return -1;
}

export function texSpans(text, { legacyCost = false } = {}) {
  return outsideCode(text, (part) => {
    let s = part.replace(/\\\(([\s\S]*?)\\\)/g, (_, tex) => `$$${tex.trim()}$$`);
    if (!legacyCost || part.includes('\\(')) return s;
    let out = '';
    for (let i = 0; i < s.length; ) {
      const m = /\b[OΘΩ]\(/.exec(s.slice(i));
      if (!m) {
        out += s.slice(i);
        break;
      }
      const start = i + m.index;
      const end = balancedEnd(s, start + 1);
      if (end < 0) {
        out += s.slice(i);
        break;
      }
      const head = { O: 'O', 'Θ': '\\Theta', 'Ω': '\\Omega' }[s[start]];
      out += s.slice(i, start) + `$$${head}(${legacyTex(s.slice(start + 2, end))})$$`;
      i = end + 1;
    }
    return out;
  });
}

// Every `$$...$$` span of `text`, for checking with KaTeX.
export const mathSpans = (text) => [...outsideCode(text, (p) => p.replace(/[^$]+(?=\$\$)|^[^$]*/g, (x) => x)).matchAll(/\$\$([^$]+)\$\$/g)].map((m) => m[1]);

// Math as plain Unicode, as `cpl_doc::plain::math` writes it: for places
// where KaTeX markup would cost more than it reads, like a 2,000-row table.
export function plainMath(tex) {
  const words = [['\\log', 'log'], ['\\ln', 'ln'], ['\\min', 'min'], ['\\max', 'max'], ['\\cdot', '·'], ['\\times', '×'], ['\\alpha', 'α'], ['\\Theta', 'Θ'], ['\\Omega', 'Ω'], ['\\le', '≤'], ['\\ge', '≥'], ['\\infty', '∞'], ['\\,', ' '], ['\\;', ' ']];
  const sup = { 0: '⁰', 1: '¹', 2: '²', 3: '³', 4: '⁴', 5: '⁵', 6: '⁶', 7: '⁷', 8: '⁸', 9: '⁹', n: 'ⁿ' };
  let s = tex.trim().replace(/\\sqrt\{([^}]*)\}/g, '√$1');
  for (const [from, to] of words) s = s.split(from).join(to);
  s = s.replace(/\^(\{([^}]*)\}|(.))/g, (_, __, group, one) => {
    const e = group ?? one;
    return [...e].every((c) => sup[c]) ? [...e].map((c) => sup[c]).join('') : `^(${e})`;
  });
  return s.replace(/\s*·\s*/g, ' · ').replace(/\s+/g, ' ').replace(/\( /g, '(').replace(/ \)/g, ')');
}

// `text` with its math spans written in plain Unicode.
export const plainSpans = (text) => text.replace(/\$\$([^$]+)\$\$/g, (_, tex) => plainMath(tex));
