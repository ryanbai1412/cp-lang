# noi10p2

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (`dream.cp`). The k best chords by a max-heap of (start, window of ends), each pop splitting its window around the end used (lines 28-35), with a `SparseTable` giving each window's best end.

## Checklist failures

- idiomatic-code/R6: `Chord(from: Int, ends: Range<Int>, at: Int)` (line 20) drops the `Note.cuts` domain, so `chord` converts back and forth (lines 25-26).
- The table keeps the index of the larger prefix with a lambda (line 17) where `SparseTable(max, ... (pre[c], c))` says it.
- The heap of split windows is the hand version of the dream's `split_best(...).take(k)`, kept because `split_best` builds its whole list before the `take`: in the friction.

## Owed rewrites

- Type `from`, `ends` and `at` as cuts and fold `(pre[c], c)` pairs with `max`.

## Language gaps

- A lazy `split_best`: in the friction.

## Dream notes

`split_best(Note.cuts, range: s => s.shift(len), best).take(k).sum()` names the k-best enumeration; the dream is the editorial.
