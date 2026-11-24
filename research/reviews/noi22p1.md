# noi22p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main 6a989896d: port, dream and friction report, not the C++. There is no dream (neither dream.cp nor a `## Dream` in the friction report), so the score is capped at 3 (research/reviews/README.md). Otherwise close to the bar: per-bit majority as the only candidate, checked against a `Counter` per store, with `Deque` stores joined small to large, all derived in the header.

## Checklist failures

- Lines 56-67: the two join branches repeat the `absorb`, the `ones` update and the `store[z]` assignment with the roles swapped; only the deque move differs (idiomatic-code/R4).
- The per-value `ones` update is written three times: in `push` (line 27), inverted inline for a pop (line 43), and summed in the join (lines 60, 66) (idiomatic-code/R4).
- Line 22 initialises `store: [Label]Store = Store.first!`, so a label not yet created by a join maps to store 0 instead of being absent (`?Store`) (idiomatic-code/R5).

## Owed rewrites

- Write the dream.
- Order the two stores once as `(big, small)`, move the deque by direction, then do one `absorb`, one `ones` sum and one `store[z] = big`.
- Give pop its own `fn` beside `push`.

## Language gaps

- None.
