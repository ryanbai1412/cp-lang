# ccc02s4hard

Score: 2/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header derives
the DP, the monotone stack bounding where t_i is the group max, and the SegTree holding min dp
and min dp + max with an assign-max action. The port is that, on std `SegTree` with an
`Action`.

## Checklist failures
- idiomatic-code/R3: `fn lower(x: ?Best, y: ?Best)` (lines 18-23) is `min` lifted over `?Best`.
  The friction named `Monoid.min.opt()` as missing; std has `opt` (std/algebra.cp), and
  `((a: Best, b: Best) => min(a, b)).opt()` runs. Bookkeeping the language already removes.
- idiomatic-code/R4: `dp` (lines 40, 51) duplicates the `low` value the SegTree already stores
  (line 53). It is read once, at line 55.
- idiomatic-code/R6: the monotone stack `taller: List<Int>` and `(0..n).rev` (lines 43-44) are
  raw positions turned into `Cut(i + 1)` at each use.

## Owed rewrites
- `join` over `((a: Best, b: Best) => min(a, b)).opt()` for both fields, and drop `lower`.
- Drop `dp` and print `st.get(Cut.first!).low!.val`.
- Iterate cuts (`Cut.before_last.rev`, as the dream does) and keep cuts on the stack.

## Language gaps
- A stock assign action (`then` at line 29): in the friction.
- Bare `min.opt()` resolves `min` at `Int` (E0302 against `?Best`): in the friction, with a run.

## Dream notes
Names the right objects (`min.opt()`, an assign action, `follow(next)`), but `Best`,
`taller.after(i)` and `.names` are never defined. A fragment, not a program (caps at 3).
