# Rebuild round 1: 40 snippet questions, answered from the spec alone

Each question: does it compile; if so what it prints, if not which
diagnostic. Diagnostic codes come from `spec/conformance/` file headers,
because no chapter in `spec/core` maps codes to meanings (hole H-DIAG-CODES).
A `?` after an answer means I had to guess; the matching hole is named.
Unless a snippet says otherwise: `domain V = 1..=4`, `var a: [V]Int = 0`,
`l, r: V`, top level, no `@!multitest`.

## Ownership and lending

Q1. `var xs = List(1, 2); let ys = xs; print(xs.len)`
No: use after move, E0608. R2 (§2 of ch5), conformance `heap-move`.

Q2. `let w = a[l..=r]; print(w.sum()); a[l] = 5`
Yes, prints the sum. `w` is dead before the write. R05.7.6, R7.

Q3. `let w = a[l..=r]; a[l] = 5; print(w.sum())`
No: E0602 (write to a source while lent). R05.7.6.

Q4. `var rows = List(List(1)); let row = rows[0]; rows.push(List()); print(row.len)`
No: `row` is a lend of `rows`, push resizes the source. E0602. R05.3.5,
R05.8.4.

Q5. `fn same(xs: List<Int>) -> List<Int> = xs`
No: returns a lent parameter without `lends`. E0603. R5, R05.7.8.

Q6. `var cnt = 0; let inc = () => cnt += 1; inc(); print(cnt); inc(); print(cnt)`
Yes, prints `1` then `2`. Captures are borrowed per call. R10, R06.2.1.

Q7. `var xs = List(3, 1, 2); xs.sort(by: x => xs.len - x)`
No: E0601 (the call holds `xs` mut and the closure reads it). R10.

Q8. `swap(mut a[l], mut a[r])` with no fact about `l != r`
No: E0601, elements may be the same. R05.8.2. With `if l != r:` it
compiles.

Q9. `fn tail(xs: [V]Int, l: V) lends [V]Int = xs[l..]`
Yes by ch5 §6 and ch7 R07.5.10, which spell `lends` with no arrow. The
grammar (§1.10 `ret`) only allows `-> lends T`. I picked "compiles";
H-LENDS-ARROW.

Q10. `let ws = read<[V]Int>(); var st = SegTree(V, op: (x, y) => if ws[x] <= ws[y]: x else: y)`
Yes per ch5 ex20 / R8 (storage dies no later than `ws`). But ch7
R07.5.10 says a lent value "never goes in a var". Also `SegTree(V, op: ...)`
does not match the `SegTree.new(op:, a:, fill:, act:, over:)` stub: there
is no positional domain parameter. ? H-LENT-IN-VAR, H-SEGTREE-DOMAIN-ARG.

## Domains, positions, cuts, ranges

Q11. `let v: V = V.first`
No: E0302, `V.first` is `?V`. R04.2.3, conformance `dom-first-not-position`.

Q12. `print(a[9])`
No: E0902, literal outside the domain's static bound. R04.2.18.

Q13. `domain V = 1..=read<1..=2e5>(); var a: [V]Int = 0; print(a[1])`
? `1` is inside every possible `V` (count >= 1), so I say yes, prints `0`.
R04.2.18 only says "outside a static bound it is a compile error"; it does
not say whether the bound is the minimum or maximum count. H-LITERAL-BOUND.

Q14. `let v = V.first!; print(v + 1)`
Yes, prints `2`: position arithmetic gives `Int`. R04.8.2.

Q15. `let v = V.first!; let u: V = v + 1`
No: E0302 (no implicit Int to position). R02.11.2, R04.8.2.

Q16. `let pre = a.scan(+); print(pre[V.first!])`
Yes, prints `0` (the fold through the first position). R07.1.6.

Q17. `domain P = 0..4; print(Int(P.cuts.last!))` and on `domain V = 1..=4`
Both print `4`: a cut's integer counts the positions before it, whatever
the base. R04.2.16. (`V.cuts.last` is `?`, R04.2.5 + R04.2.3.)

Q18. `for j in ..V.cuts(2): print(Int(j))`
Yes, prints `0` then `1`: `..i` on a cut walks cuts. R04.4.8, R04.4.2.

Q19. `let s = V.cuts(1)..V.cuts(3); print(a[s].sum())`
No: E0302. A stored cut range never becomes a piece; only the literal
does. R04.4.5, conformance `closed-cut-range-not-piece` (closest case).

Q20. `print((0..=10).last_true(k => k * k <= 50) ?? -1)`
Yes, prints `7`. R04.4.19.

Q21. `let r = V.range; print(Int(r.mid))`
Yes, prints `2` (two positions before the middle cut). R04.7.3.

Q22. `print((1..=5).bsearch(x => x >= 3))`
? The answer space is an Int range, "positions and cuts are both Int"
(ch4 Ranges, Methods). So the cut is an `Int`. Which Int: the count of
elements before it (2) or the first true element (3)? `Int(c)` for cuts of
`1..=n` is v-1 for `v.l`, so I say `2`. H-INT-RANGE-CUTS.

## Optionals

Q23. `let d: ?Int = None; if d + 1 < 5: print("x")`
No: E0308, ordering does not lift. R02.4.5.

Q24. `var dist: [V]?Int = None; dist[V.first!] = 3; dist[V.last!] min= dist[V.first!] + 2; print(dist[V.last!] ?? -1)`
Yes, prints `5`: a `None` target takes the right side. R02.4.6. The
scalar initializer `= None` filling an array is from conformance
(`anon-range-array`), not stated in R02.8.3. H-ARRAY-SCALAR-FILL.

Q25. `let x: ?Int = None; print(x)`
No: `?T` is not `Print`; E0606 (closest conformance `bool-not-print`).
R10.5.

Q26. `let x: ?Int = Some(3); if x? > 2: print(1)`
No: E0803, `?` in a condition. R02.4.7.

Q27. `let x: ?Int = None; print(x.map(v => v * 2) ?? 0)`
Yes, prints `0`. R02.4.1, R02.4.2.

Q28. `var x = None; x = 3`
No: E0806, inference does not look at later statements. R02.10.1.

## Widths and numbers

Q29. `print(-7 / 2, -7 % 2)`
Yes, prints `-4 1`: floor division, non-negative `%`. R01.5.2.

Q30. `let x: 0..=10 = 11`
No: E0902 (constant range store), conformance `const-range-store`.
R03.1.4.

Q31. `var x: 0..=10 = 5; x += 10` in a checked run
Compiles; traps R0201 in checked builds, truncates in release. R03.1.4,
R09.3.3.

Q32. `let x: U8 = 3`
No: E0901, there is no `U8`. R03.1.5.

Q33. `print(2 ** 3 ** 2, -2 ** 2)`
Yes, prints `512 -4`. R01.5.1 (right-assoc, unary minus below `**`).

Q34. `if 0 <= 3 < 5 == true: print(1)`
No: E0111, a chain containing `==`. R01.5.3.

## Enums and patterns

Q35. `enum Dir = L | R | U | D; var c: [Dir]Int = 0; c[Dir.U] += 1; print(c[Dir.U])`
Yes, prints `1`: a payload-free enum is a domain. R02.7.3. (The scalar
fill is again H-ARRAY-SCALAR-FILL.)

Q36. `enum Dir: N, E, S, W` (spelling used in ch4's opening example)
? The grammar has `enum X = A | B` or `enum X:` followed by an indented
variant list; `enum X: A, B` on one line is in neither. I say syntax
error. H-ENUM-ONE-LINE.

Q37. `enum E = A | B; let e = E.A; match e: A => print(1)`
No: E0314, non-exhaustive. R02.7.4.

Q38. `match 5: big => print(big)`
Yes, prints `5`: a lowercase name binds. R01.3.3.

## I/O, contexts, generators, closures

Q39. `print(true)`
No: E0606, `Bool` is not `Print`; fix `print(yes(true))`. R10.3.3.

Q40. A block whose second line is a bare `with`:
`fn f():` / `    print(1)` / `    with checks(overflow: off)` / `    print(2)`
No: E0802, a bare `with` line must come first in its block. R09.1.1.

Q41 (bonus, generators). `fn evens(n: Int) -> yields Int: for i in 0..n if i % 2 == 0: yield i` then `print(evens(5).sum())`
No: syntax error. A one-line body after `:` holds only simple statements
(`block := ':' simple_stmts NL`, R01.2.3), and `for` is compound. Indented,
it prints `6` (R06.3.1, R07.9.1).

Q42 (bonus, closures). `fn counter() -> (() -> Int): var c = 0; () => c`
No: E0603, a closure over a local can't be returned. R05.10.2.
