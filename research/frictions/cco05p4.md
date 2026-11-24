# Friction Report: cco05p4

The v0.2 port read `n: Int` and `[0..n]Int`, rebuilt the prefix table from a comprehension, and
called std `is_prime` per window, which rebuilds its base list on every call: 19 s at `--max`
against 1.5 s for the accepted C++. Main's fix sieved the odd sums once into a Bitset with
`while` loops. Now the reads carry the statement bounds (`1..=1e4`, `[n]`), `prefix(a)` takes the
read directly, the sieve is two `for .. by` loops with the prime test in the filter, and the
leftmost window of each length is `(0..=n - len).find(..)`.

## std `is_prime` rebuilds its constant bases per call
- **Where:** the odd-only Bitset sieve stands in for `is_prime(sum)`.
- **Ideal:** std `is_prime` with its Miller-Rabin bases as a constant, fast enough per window;
  repro sent to Perf A.

## No odd-only or bit-packed sieve in std
- **Where:** `Bitset<Half>` with bit k for 2k + 1, so 1e8 fits in 16 MB; std `Sieve(n)` keeps a
  smallest-factor table of `n + 1` Ints.
- **Ideal:** `PrimeBits(top)` (odd-only, bit-packed) with `.contains(x)`.

## Dream
```cp
@multitest
let n: 1..=1e4 = read()
let a: [n]1..=1e4 = read()
let pre = prefix(a)
for len in 2..=n:
    if let l = (0..=n - len).find(l => is_prime(pre.cut(l + len) - pre.cut(l))):
        print("Shortest primed subsequence is length {len}:", a[l..l + len])
        return
print("This sequence is anti-primed.")
```

- Tokens: port 241, original 539
- Max input (`gen.py --max`): port 0.95 s, 9744 KB, original 1.29 s, 15744 KB
