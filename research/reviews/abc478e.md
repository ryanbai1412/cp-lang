# abc478e

Score: 3/5
Dream: 5/5

Cold review of `port.cp`, without the C++; there is no `dream.cp`, and the friction's dream is
the port as written. `scc` on the condition graph and `any(t == 1 and comp[u] == comp[v] ...)`
give the contradiction test. One relaxation pass over arcs sorted by source component, in the
topological order `scc` returns, gives the longest path. Values are typed `[C]1..=n`.

## Checklist failures
None found. The only cap is the missing `dream.cp`: a port with no dream scores at most 3.

## Owed rewrites
- Commit the friction's dream as `dream.cp` once `corpus.py verify` passes on it. It is the
  port, so it should pass as is; that lifts the cap.

## Language gaps
None.

## Dream notes
Reads like the editorial.
