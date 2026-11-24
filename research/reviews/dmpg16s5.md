# dmpg16s5

Score: 2/5
Dream: none

Cold review of `port.cp` (no `dream.cp`, no friction `## Dream`, so capped at 3). The header derives the unrolled circle and the window of at most k cuts; the code keeps that window by hand.

## Checklist failures
- idiomatic-code/R3: lines 13-21 are a hand-kept monotone deque (`while let (_, last) = window.last and last >= pre: window.pop()`, `if window.first!.cut < cut + 1 - k: window.pop_front()`). std has the sliding fold queue `Window(min)` (`std/window.cp`), and the friction reports neither the hand-rolled queue nor a run showing std is too slow.
- idiomatic-code/R6: cuts `0..2 * n` and slices `0..n` are both raw `Int`, crossed by `taste[cut % n]` (line 20).

## Owed rewrites
- Keep the last k prefix sums in `Window<Int, _>.new(min)` (push each prefix, pop the one k cuts back) and print the best `pre - w.all!`.
- Write a dream naming the windowed minimum of prefix sums.

## Language gaps
None shown.

## Dream notes
No dream.
