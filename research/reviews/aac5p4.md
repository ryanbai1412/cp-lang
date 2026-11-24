# aac5p4

Score: 2/5
Dream: 3/5

Cold review of `port.cp` against `dream.cp`, without the C++. The header derives the bound
M >= h and the parity rule. The construction, though, is a C++ two-pointer walk, and none of the
port/dream gap is in the friction report, which covers only the corpus checker.

## Checklist failures
- idiomatic-code/R2: the walk (lines 50-62) is `while true` with two inner
  `while i > 0 and not ahead[i]: i -= 1` skips over a `List.filled(k + 1, false)` flag array
  indexed by raw sizes. The dream splits the sizes into `fwd` and `back` lists and walks those.
- idiomatic-code/R4: the n = 3, x = 2 special case is hard-coded on line 25 (`moves = [1, -2]`)
  ahead of the general cases. The dream derives it as `e == n` (nothing above h fits).
- Unreported: none of the above is in `research/frictions/aac5p4.md`, and the dream's lines
  only use `List` and `push`, so nothing in the language blocks them.

## Owed rewrites
- Take the dream's `fwd`/`back` split and the `e == n` case, then report anything that does
  not compile.

## Language gaps
- The corpus suite compares tokens instead of running the problem's `@check`: in the friction
  (tools).

## Dream notes
- idiomatic-code/R7: `let d = if ...: i += 1; fwd[i - 1] else: j += 1; -back[j - 1]` hides two
  cursor bumps inside an expression (caps at 3). Two queues
  (`fwd.pop_front()`/`back.pop_front()`) or iterators would make each step a value.
- The `@check` with the closed-form `least` is in the port but missing from the dream.
