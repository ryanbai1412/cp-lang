# ioi10p3

Score: 4/5
Dream: 5/5 (the friction's `## Dream`)

Re-ported in v0.3 style.

## Cold review (HEAD)

Port 4/5, dream 5/5. Binary search on the answer with `first_true`, and a 2D prefix count of `rank <= x` tested over every window with `windows`: the header in seven lines. Checklist miss: the prefix is filled with a nested `set` loop (lines 18-20) where the dream builds it from one table; the table form is 1.63x the C++ at --max ([friction](../frictions/ioi10p3.md), owner Perf B).
