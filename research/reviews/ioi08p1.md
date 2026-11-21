# ioi08p1

Score: 3/5

Baseline review, cold, on main 15cc353a, after 14cdbd55 moved it onto `std.strings.Trie`. There is no dream, so the score is capped at 3. The header (lines 1-11) states the walk and why the longest word goes last, and the body follows it.

## Checklist failures

- Line 27 keeps `on_longest` as a list parallel to the trie's nodes; marking the path is the fact the header names, so a set of nodes or a per-node map says it without the parallel array.
- `Char(c.label)` (lines 40, 42) converts the letter domain back by hand twice; `let _ = trie.add(...)` (line 25) discards a result.

## Rewrites owed today

- Write the dream.

## Gaps that need the language

None found; no friction report is needed.
