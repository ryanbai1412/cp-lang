# egoi23p7

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port. This review found the port to be a translation (score 2: recursion, globals, raw component ids, a `Set<Comp>` rebuilt at every visit). The port is now the verified dream. It agrees with the old port on 30 gen.py cases under `checker.py` and passes its 21 package cases.

The program reads in the statement's terms:
- the required arcs are a `Rooted.try_from_parents` forest or NO;
- a block may not hang under k when its ban tally reaches `forest.size(k)`;
- `search` builds a spanning tree in the complement graph, scanning only unreached blocks.

## Checklist failures

- `(root, seen[s]) = (s, true)` and `(seen[c], below[c]) = (true, Some(k))` pack two unrelated stores into one tuple assignment. Two lines read better.
- `mark[c] == Some(k)` marks with the current block to skip clearing the array between blocks. This is a valid trick, but nothing names it.
- The duplicate-ban filter keeps `bans` and `banned_by` side by side (friction, low).

## Gaps that need the language

- None. The friction's grouped-read helper would shorten the input handling, but it is not needed.
