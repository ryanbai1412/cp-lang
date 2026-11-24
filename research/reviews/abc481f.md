# abc481f

Score: 3/5
Dream: 4/5

Cold review of `port.cp`, without the C++; there is no `dream.cp`, and the friction's dream is
the port as written. `value: [V]?Z` marks unreached vertices as `None`. Layers are ordered
lists of groups sharing a smallest digit string, each group buckets its unvisited out-arcs into
`[1..=9]List<V>`, and the buckets in order are the next layer.

## Checklist failures
- idiomatic-code/R4 (minor): unreached is tested twice, once while bucketing
  (`if value[a.to].is_none`) and again when the bucket is emptied (`if value[v].is_none`). Both
  are needed, because a vertex can enter several buckets of one group, but the header doesn't
  say why the second test exists.
- Capped at 3 for the missing `dream.cp`.

## Owed rewrites
- One comment on the second `is_none` (a vertex may sit in several buckets; the first
  digit wins).
- Commit the dream as `dream.cp` after `corpus.py verify` passes.

## Language gaps
None.

## Dream notes
The same as the port, so the same R4 note; 4/5.
