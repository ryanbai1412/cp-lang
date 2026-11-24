# Friction report: sdoi17r1p3

There is no dream.cp; the ideal is below. The port takes entry 0 of the n-th power of a residue-count vector under cyclic convolution mod p, once for 1..=m and once for its non-primes. The header said "segmented sieve", but the port had a hand sieve of a 2e7-entry `List<Bool>`. This review replaced it with std's `primes_in(2, m + 1)`, the segmented sieve the header names: 38 MB against the 128 MB limit, 0.24 s.

## Dream

```cp
domain Residue = 0..p      // with Residue + Residue wrapping mod p
fn convolve(a: Counts, b: Counts) -> Counts = [Residue] r => Residue.sum(i => a[i] * b[r - i])
for q in primes_in(2, m + 1): plain[Residue.of(q)] -= 1
```

## Cyclic index arithmetic goes through `idx` and `% p`
- **Where:** line 23: `c[Residue((i.idx + j.idx) % p)] += a[i] * b[j]`, and line 26: `r.idx == 0`, `1 <= r.idx and r.idx <= m % p`.
- **What felt wrong:** the residues mod p are a group, but each sum leaves the domain for `Int`, reduces by hand and re-enters.
- **Goal it hurts:** express.
- **Ideal code:** the dream's wrapping `Residue + Residue` (a cyclic domain), and `Residue.of(q)` for "q's residue".
- **General fix:** a cyclic `domain` form (or `Mod<p>` usable as an index domain) whose arithmetic wraps. Owner: core/spec.
- **Severity:** low.

## Measurements

- **Tokens:** port 246, accepted C++ 996 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.24 s / 37 MB, accepted C++ 0.28 s / 22 MB, with the same output.
- **Emitted C++:** 16542 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
