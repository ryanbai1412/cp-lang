// Source: Codeforces 1909G (Pumping Lemma), https://codeforces.com/contest/1909/problem/G
// Problem: lowercase strings s, t of lengths n < m <= 1e7. Count the triples (x, y, z) with
//   s = x + y + z and t = x + y^k + z for some integer k.
// Solution: t is s with delta = m - n characters inserted, delta = (k - 1)|y|. x lies in the
//   common prefix, z in the common suffix, so y lies in the `width`-wide window from `start` where
//   s matches t from both ends; each start of a length-a y there gives a triple. The inserted
//   block t[start..start + delta] must be a power of y: a | delta and the block is a-periodic. By
//   Fine-Wilf those a are the multiples of g dividing delta, g = the primitive period p if
//   p | delta, else delta. Sum width - a + 1 over them: O(m).

let (n, m): (1..=10_000_000, 2..=10_000_000) = read()
let (s, t): (Str, Str) = read()
let delta = m - n
let start = n - s.common_suffix_len(t)
let width = s.common_prefix_len(t) - start
if width <= 0: return print(0)

let p = period(t[start..start + delta])
let g = if delta % p == 0: p else: delta
print([width - a + 1 for a in (g..=min(width, delta)).step(g) if delta % a == 0].sum())
