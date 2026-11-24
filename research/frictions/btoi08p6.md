# btoi08p6: friction report

Port: `corpus/problems/btoi08p6/port.cp`. There is no `dream.cp`; the dream is below.

## Measurements

- Tokens: port 269, accepted C++ 882.
- Emitted C++: 15065 bytes.
- `--max` (n = 20, median of 15): port 67.8 ms / 10.5 MB, accepted 143.5 ms / 18.6 MB (0.47x time).
- Output: accepted by `checker.py` and identical to the accepted C++.

## Dream

```cp
let n: 1..=20 = read()
domain C = 0..n
type Colours = Subset<C>
let a: [C]0..=1e8 = read()
let b: [C]0..=1e8 = read()

// colour set s fails every (L, R) with L <= its left sum and R <= the right sum off it
let boxes = [Colours] s => (l: a.sum(over: s), r: b.sum(over: s.complement))
    .sorted(by: x => x.l)

let candidates = boxes.windows(2).map(w => (l: w[0].l + 1, r: max(x.r for x in after(w)) + 1))
let best = min(c for c in candidates if c.l <= a.sum() and c.r <= b.sum(), by: c => c.l + c.r)!
print(best.l)
print(best.r)
```

The dream states the boxes as a `Subset<C>` table of records and the suffix maximum as
a reduction; it is unverified (the `after` and `sum(over:)` spellings are not probed).

## Friction 1: two sums packed into one Int to fit the memory limit

- Where: `let shift = 1 << 32`, `boxes[s + (1 << c.ord)] = boxes[s] + a[c] * shift - b[c]`,
  `box / shift + 1`, `box % shift`.
- What felt wrong: each colour set's (left sum, right sum off it) is stored as
  `l * 2^32 + r` so 2^20 of them sort as one `List<Int>` within 32 MB. That is the
  accepted C++'s trick, and the comment has to explain it; the subset enumeration is also
  done with raw `1 << c.ord` masks instead of `Subset<C>`.
- Goal it hurts: R2 (translated), R3 (`Subset`), R5 (two bounded quantities as one
  unbounded Int).
- Ideal code: `List<(l: 0..=2e9, r: 0..=2e9)>` sorted by `l`, built over `Subset<C>`.
- General fix: port edit. A fresh probe shows the record already narrows:
  `var xs: List<(l: 0..=2_000_000_000, r: 0..=2_000_000_000)>` emits
  `std::vector<std::tuple<uint32_t, uint32_t>>`, 8 bytes per box, the same as the packed
  `Int`, so the packing buys nothing today.
- Severity: medium (the packing is the least readable part of the port).
- Owner: idioms.

## Friction 2: "better than the best so far" is spelled with an `if let ... else: true`

- Where: `(if let p = best: l + r <= p.l + p.r else: true)`.
- What felt wrong: std has `?T.is_none_or(p)` (std/opt.cp:30), which says this exactly.
- Goal it hurts: R8.
- Ideal code: `best.is_none_or(p => l + r <= p.l + p.r)`.
- General fix: port edit.
- Severity: low.
- Owner: idioms.
