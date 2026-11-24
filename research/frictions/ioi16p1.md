# Friction report: ioi16p1

## Dream

```cp
let order = I.sort(by: i => (w[i], i))
let window = order.windows(shrink_while: ws => ws.sum(of: i => w[i]) > u)
    .first(ws => ws.sum(of: i => w[i]) >= l)
print(window.map(ws => ws.len) ?? 0)
print(window ?? [])
```

## Two pointers over a list are spelled with `Int` indices

- **Where:** `var (lo, total) = (0, 0)`, `total -= w[order[lo]]`, `order.take(hi + 1).drop(lo)`.
- **What happened:** the window is two `Int`s into `order` plus a running sum kept by hand; the
  sum and the window can drift apart. Nothing in std grows and shrinks a window.
- **General fix:** a std sliding window over a sequence (grow, shrink while a predicate holds,
  with a running fold), used by every two-pointer port.
- **Goal it hurts:** hard to get wrong. **Severity:** low.

## Grader task

The accepted C++ is the grader function `find_subset`; the port plays the sample grader on stdin
(see `research/frictions/ioi22p5.md` for `@grader` builds).

## Measurements

Contestant-written tokens: port 252, original 241 (`corpus/audit.py` count). Emitted C++: 18765 bytes (`cpl build`, judge `generic`).
The accepted C++ is a grader function with no main, so the max input is timed only by `corpus/generators.py`.
