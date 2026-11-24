# coci25c3p5

Score: 3/5
Dream: none

The port counts all vertical and horizontal segments, then subtracts shared heights between adjacent columns. The header explains the rational-spacing argument (lines 7–10), and `fn shared(l: Fan, r: Fan) -> Int:` (line 20) makes the final sum read like that argument (idiomatic-code/R2). Statement bounds are typed at lines 14–17 (idiomatic-code/R5); the score is capped by the missing dream.

## Checklist failures
- The required dream is missing: there is neither `dream.cp` nor a `## Dream` section in the friction report (porting process, dream-first requirement).

## Owed rewrites
- Supply a dream stating the shared-height invariant and expressing the adjacent-pair sum; no code rewrite is otherwise owed.

## Language gaps
- Performance (in the friction): the reported max-input times are 0.08 s versus 0.04 s, exceeding the 1.1x target (idiomatic-code/R10). The report records the slowdown but does not establish its cause; line 26 is the aggregate computation: `print(n + 1 + sum(b[i] + 1 for i in Fan) - sum(shared(l, r) for r in Fan if let l = r.prev))`.

## Dream notes
None: there is no `dream.cp` and no `## Dream` in the friction report, so the port is capped at 3.
