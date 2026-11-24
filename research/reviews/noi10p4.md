# noi10p4

Score: 3/5
Dream: none

Port 3/5, dream none. The friction report has no `## Dream` section and there is no `dream.cp`, so the port is capped at 3. Deadlines tighten along `topo(g)` reversed (lines 21-23) and each plane's earliest slot fills the others as late as their deadlines allow (lines 28-39), as the header argues.

## Checklist failures

- `earliest` returns `?Int` (line 28) and every caller unwraps it (line 41), though a valid schedule always exists.
- `due_at: [1..=n]Int` (line 31) is an anonymous slot axis.

## Owed rewrites

- Add a `## Dream`; return `1..=n` and name the slot domain.

## Language gaps

None.

## Dream notes

None.
