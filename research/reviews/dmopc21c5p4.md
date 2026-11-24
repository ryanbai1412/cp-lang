# dmopc21c5p4

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

A walk around a cycle of 1..=n by steps `up` and `-down`, where `up + down` is a prime total found by Goldbach's split.

## Checklist failures

- `n == 5 or n == 7 or (n % 2 == 0 and n != 6)` picks the total with special cases and no comment.
- `Pos(Int(at) + up)` and `Pos(Int(at) - down)` convert through `Int` on every step; a cyclic step on the domain would say it.
- `walk` is pushed by a `for _ in 0..n` loop carrying `var at`, an iterate-n-times written by hand.
- `print(3, 1, 4, 2)` hard-codes the n = 4 answer.
