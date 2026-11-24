# Friction Report: noip14p6

## 1. Reading a huge integer modulo several primes
- **Where:** `fn residue<P>(s: Str) -> Mod<P>` folds the digits and the sign by hand, once per
  modulus, from the coefficients kept as `Str`.
- **What felt wrong:** "this decimal text, mod P" is the whole reading step; `read<Mod<P>>()`
  does it for one modulus, but `Str.parse<T: Read>` is still `todo`, so the same text cannot be
  read again under a second modulus.
- **Goal it hurts:** express.
- **Ideal code:** `digits.map(s => s.parse<Z>()!)`, as in the dream.
- **General fix:** implement `Str.parse<T: Read>` by reading from the string.
- **Severity:** medium.

## Measurements
The port evaluated all m candidates under 2^61 - 1 (O(n m), 0.91 s natively and 1003 ms under
`cpl test` against the AC's 0.80 s). Sieving x by three primes near 10^4 first leaves only a
handful of candidates: `--max` is 0.06 s and 4.0 MB natively (AC 5445671: 0.85 s, 5.5 MB) and
91 ms under `cpl test`. 30/30 stress cases agree with the v0.2 port and with the AC, and eight
m <= 10^6 seeds agree with the AC.

## Dream
The ungated ideal (does not check today):

```cp
let (n, m): (1..=100, 1..=1e6) = read()
let digits: [0..=n]Str = read()
// f(x) = f(x mod p) (mod p): each small prime rules out all but n / p of the x
let sieves = primes_in(10_000, 10_100)[..3].map(p => (p:, root: [digits.map(s => s.parse<Mod(p)>()!).eval(r) == 0 for r in 0..p]))
let exact = digits.map(s => s.parse<Mod<2305843009213693951>>()!)
let roots = [x for x in 1..=m if sieves.all(s => s.root[x % s.p]) and exact.eval(x) == 0]
print(roots.len)
print(roots, sep: "\n")
```
