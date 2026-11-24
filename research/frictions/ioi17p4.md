# Friction report: ioi17p4

## Dream

```cp
@interactor fn judge(i: Box) -> (l: Int, r: Int)       // the hidden kinds stay in the interactor
var widest = 0
let ask = memo((i: Box) => judge(i).also(a => widest max= a.l + a.r))
fn search(span: Range<Box>, before: Int, after: Int) -> ?Box
let probe = Box.take(450).find(i => ask(i).l + ask(i).r == 0)
print(probe ?? search(Box.all, 0, widest)!)
```

## The judge lives in the solution

- **Where:** the `seen` / `rarer_left` / `judge` block.
- **What happened:** the port reads the hidden kinds and answers its own queries, as the v0.2 port
  did; the real contract is a grader function `find_best` calling `ask`. A file can't hold the
  solution and its interactor side by side and run them as two parties locally.
- **General fix:** `@interactor` beside the solution, run by `cpl run` / `cpl stress` as the
  other party (see `research/frictions/ioi22p5.md` for `@grader` builds).
- **Severity:** medium.

## Memoising an asked value is spelled with a flag table

- **Where:** `var asked: [Box]Bool`, `fn ask`.
- **What happened:** there is no `memo` wrapper and no `Int.also` / tap (`E0203: an Int has no
  method also`), so the side effect of a first ask is an `if not asked[i]` block.
- **General fix:** a std `memo` over a domain-keyed function, and `also` on any value.
- **Severity:** low.

## A domain's first k positions

- **Where:** `(Box.first!..=Box(min(n, 450) - 1))`.
- **What happened:** `Box.take(450)` would say it; the port builds the range from `Int`.
- **Severity:** low.

## Measurements

Contestant-written tokens: port 421, original 312 (`corpus/audit.py` count). Emitted C++: 14369 bytes (`cpl build`, judge `generic`).
The accepted C++ is the grader function `find_best` with no main, so the max input is timed only by `corpus/generators.py`.
