# ioi08p1

Score: 4/5
Dream: 4/5

Baseline review, cold, on main 15cc353a, after 14cdbd55 moved it onto `std.strings.Trie`. There is no dream, so the score is capped at 3. The header (lines 1-11) states the walk and why the longest word goes last, and the body follows it.

## Checklist failures

- Line 27 keeps `on_longest` as a list parallel to the trie's nodes; marking the path is the fact the header names, so a set of nodes or a per-node map says it without the parallel array.
- `Char(c.label)` (lines 40, 42) converts the letter domain back by hand twice; `let _ = trie.add(...)` (line 25) discards a result.

## Rewrites owed today

- Write the dream.

## Gaps that need the language

None found; no friction report is needed.

## Cold review (HEAD)

Dream 4/5 (new; port = dream), port 4/5, rescored cold on current main. Both check with zero errors. The header (lines 1-11) derives why the longest word goes last. The body is that rule: `Trie` from `std.strings`, the longest word's path as a `Set`, and a walk that defers that child.
- Minor: the `longest` loop (lines 21-25) is `words.max(by: w => w.len)!`, which I probed. Trie node handles are raw `Int` (`on_longest: Set<Int>`, line 27), because that's `Trie`'s API.
