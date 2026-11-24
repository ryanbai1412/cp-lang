# cco23p4 — Flip it and Stick it (CCO 2023 P4)

Port: `corpus/problems/cco23p4/port.cp`. The old port kept the string twice as `List<Int>`
(as read and complemented), had `count1`/`count2`/`count3` index loops, and an eight-way
`p0`/`p1`/`p2` ladder over the bits of T. The rewrite relabels S and T so T starts with
`'0'` (`plain`), counts T in S with `windows`, takes the zero gaps between ones with
`split('1')`, and matches the five pattern shapes once.

## Frictions

- **A string's length bound is not a type.** `let s: Str = read()` reads S, but
  `|S| <= 2e5` and `|T| <= 3` can only live in the header comment.
- **A window compares to a `Str` only after a copy.** `w.windows(p.len).count(x => Str(x) == p)`
  builds a string per window; a slice of a `Str` doesn't compare with a `Str` directly.

## Dream

```cp
let s: Str<..=2e5> = read()
let t: Str<1..=3> = read()
let hits = w.windows(p.len).count(p)
```

## Measurements

Max case (|S| = 2e5, T = "000", zeros at the feasible limit): port 0.01 s, accepted C++ 0.01 s,
same output.
