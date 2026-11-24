# cco23p6

Score: 4/5
Dream: 3/5

Cold review against the friction's `## Dream`. It reads as the editorial: `Run(made, singles, pairs)` with `join` matching left singles to right pairs (lines 19-23), a `SegTree<Len, Run>` (line 28), and the closing formula `made + (pairs - made) * 2 / 3` (line 42) derived in the header (lines 9-10). 0.11 s against the C++'s 0.19 s.

## Checklist failures

- None beyond the reported gap (fresh run in the friction).

## Owed rewrites

- None in the port.

## Language gaps

- A `SegTree` whose leaves are a fn of other tables (`SegTree.of(join, leaf)`), removing `refresh`: in the friction.

## Dream notes

- A fragment judged as the port with `SegTree.of`. It writes `sign * (have[i] / 2)` twice (R4, the same formula twice), where the port binds `let two = have[i] / 2` (line 31): capped at 3. Bind it once and the dream is a 4.
