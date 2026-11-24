# ioi14p4: Gondola

## Dream

```cp
@grader fn valid(seq: [Slot]Number) -> Bool
@grader fn replacement(seq: [Slot]Number) -> List<Number>
@grader fn countReplacement(seq: [Slot]Number) -> Z
let rotation = seq.originals().map(i => (seq[i] - i) % n).unique()  // ?Int, None when they disagree
```

## Three grader functions become one stdin task switch
- **Where:** `task` and the final `if task == 1 ... elif ... else`; the judge
  calls `valid`, `replacement` and `countReplacement` separately, and
  `orig.cpp` adds a main that dispatches on the task number.
- **Severity:** high.

## "All equal" over a list is `all(s => s == first!)`
- **Where:** `ok`: the rotation every surviving original agrees on. A
  `unique()`/`all_equal` returning the shared value would replace `shift` and
  half of `ok`.
- **Severity:** low.

## One accepted C++ is wrong on task 1
- **Where:** submission 4684550's `valid` checks the rotation of every number
  (even replaced ones) and never checks distinctness: on `1 / 6 / 10 12 4 5 6 8`
  it prints 0 where the answer is 1. `orig.cpp` wraps 4684567 instead.
- **Severity:** medium (evidence only).

## Task 2 has many answers
- **Where:** `checker.py` replays a replacement sequence backwards to a
  rotation of 1..n; the port keeps the old port's choices (rotation `n - 1`
  when none survives, the largest slot absorbs the gaps).
- **Severity:** low.
