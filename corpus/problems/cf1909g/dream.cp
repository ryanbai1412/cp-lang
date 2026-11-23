// Codeforces 1909 G: Pumping Lemma (dream).
// s = x + y + z and t = x + y^k + z, so t is s with delta = m - n extra
// characters, delta = (k - 1) |y|. x lies in the common prefix and z in the
// common suffix, so y lies in the window of `width` characters from `start` where
// s matches t from both ends, and every start of a length-a y inside it gives a triple. Pumping y
// inserts the delta-block t[start..start + delta], which must be a power of
// some y: its length a divides delta and the block is a-periodic.
// Fine-Wilf: a period dividing delta is a multiple of the primitive period p,
// and exists below delta only if p divides delta. So the lengths are the
// multiples of g dividing delta, g = p if p | delta else delta.

let (n, m): (1..=10_000_000, 2..=10_000_000) = read()
let (s, t): (Str, Str) = read()
let delta = m - n
let start = n - s.common_suffix_len(t)
let width = s.common_prefix_len(t) - start
if width <= 0: return print(0)

let p = period(t[start..start + delta])
let g = if delta % p == 0: p else: delta
print([width - a + 1 for a in (g..=min(width, delta)).step(g) if delta % a == 0].sum())
