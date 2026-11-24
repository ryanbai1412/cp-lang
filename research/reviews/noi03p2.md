# noi03p2

Score: 3/5
Dream: 4/5

Port 3/5, dream 4/5 (the friction's `## Dream`, a `Rope` fragment). A piece table cut where edits land, with start offsets searched by `bsearch` (lines 14-33): a sound rethink for at most 4000 edits.

## Checklist failures

- idiomatic-code/R7: `let (i, j) = (cut(cursor), cut(cursor + n))` (line 52) evaluates two mutating calls in one tuple, and `i` is only right because the first cut happens first.
- idiomatic-code/R3: `reindex` (lines 18-20) rebuilds the offsets as a hand prefix-sum loop after every edit.
- The commands are an `if`/`elif` chain on strings (lines 37-61), where `match cmd:` on string literals works (checked with `cpl run`).
- The header says pieces are moved, "never copying them" (line 9), but a cut copies both halves with `to_str()` (lines 29-31).

## Owed rewrites

- Two statements for the cuts, a `match` for the commands, `starts` from a prefix scan of the piece lengths, and a header that counts the split copy.

## Language gaps

- No growable sequence with O(log n) insert, range erase and slice, and no `List` range erase (line 53): in the friction.

## Dream notes

`Rope<Char>` with `insert`, `erase` and slicing is the original's structure and the right dream; it shows three lines and no command dispatch.
