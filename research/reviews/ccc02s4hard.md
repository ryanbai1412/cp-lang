# ccc02s4hard

Score: 2/5
Dream: 3/5

Cold review of `port.cp` against the friction's `## Dream`, without the C++. The header derives
the DP, the monotone stack bounding where t_i is the group max, and the SegTree holding min dp
and min dp + max with an assign-max action. The port is that, on std `SegTree` with an
`Action`.

## Checklist failures
- idiomatic-code/R3: `fn lower(x: ?Best, y: ?Best)` (lines 18-23) is `min` lifted over `?Best`.
  The friction names `Monoid.min.opt()` as the ideal, and std has it (`opt` in
  std/algebra.cp). Bookkeeping the language already removes.
- idiomatic-code/R4: `dp` (lines 40, 51) duplicates the `low` value the SegTree already stores
  (line 53). It is read once, at line 55.
- idiomatic-code/R6: the monotone stack `taller: List<Int>` and `(0..n).rev` (lines 43-44) are
  raw positions turned into `Cut(i + 1)` at each use.

## Owed rewrites
- `join` over `min.opt()` for both fields, and drop `lower`. Correct the friction.
- Drop `dp` and print `st.get(Cut.first!).low!.val`.
- Iterate cuts (`Cut.before_last.rev`, as the dream does) and keep cuts on the stack.

## Language gaps
- A stock assign action (`then` at line 29): in the friction.

## Dream notes
Names the right objects (`min.opt()`, an assign action, `follow(next)`), but `Best`,
`taller.after(i)` and `.names` are never defined. A fragment, not a program (caps at 3).
