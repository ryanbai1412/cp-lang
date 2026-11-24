# noi07p1

Score: 4/5
Dream: 4/5

Port 4/5, dream 4/5 (`dream.cp`). Floyd-Warshall that also counts shortest paths over `?(d, ways)` (lines 13-26), and `share` states when v lies on a shortest s-t path and what fraction it carries (lines 27-29): the header's argument.

## Checklist failures

None beyond the friction: the `if s == t: Some(...) else: None` table (line 13) and the ordered-pair comprehension (line 33) are reported with quoted errors for `Bool.then` and ordered pairs.

## Owed rewrites

- `Some(st) if st.d < d => Some(st)` (line 24) rebuilds the entry it keeps; `if d < st.d` / `elif d == st.d` updates read shorter.
- The friction's "ideal code" lines (`.then`, `pairs_ordered`) are not what `dream.cp` writes; the two should agree.

## Language gaps

In the friction.

## Dream notes

Reads as the editorial; the final sum folds a long lambda into `print`, where the port's `share` is clearer.
