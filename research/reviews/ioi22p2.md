# ioi22p2

Score: 4/5
Dream: 3/5

Cold review: the port is better than its dream.cp: `Bag`, `Act` and `Step` are enums, `settle` and `descend` name the two outcomes (lines 52-61), and the strategy table is a comprehension over `Count`. The state numbering and the tree sizes are still hand arithmetic, and both are reported. Port score provisional until the dream is rewritten.

## Checklist failures

- R2: `size` is built by `push` over `fanout.rev` then `reverse()` (lines 23-25), a C++ loop where a suffix scan of `fanout` says it.
- R5: counts whose walk stops early get `Act.Write(0)` (line 71), a filler state; the comment explains it, but no type says the cell is unreachable.

## Owed rewrites

- `size` as a reversed scan (`fanout.rev.scan(2, (s, b) => b * s + 2).rev`) instead of push-then-reverse.

## Language gaps

- The grader's integer action codes, states as a ragged domain, nested enum patterns (E0314), and the hand interval tree: all in research/frictions/ioi22p2.md.

## Dream notes

- dream.cp reads like the C++: `bag` is an `Int` (`t % 2`), `smaller(bag: Int, mine: Bool)` encodes the grader's -1/-2, `first_state` and `rsize` are push-built lists, and each row is pushed in a `for v in 1..=n` loop. The friction's `## Dream` sketch (`SplitTree`, `domain State`, `[State](opens: Bag, act: [1..=n]Act)`) is the dream it should be; rewrite dream.cp along it.
