# ioi13p5 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi13p5.md).

Reads like the editorial: toys bucketed by the first weak robot strong enough, a feasibility
check with a size heap, and `Minutes.first_true` as the binary search. `Desc<Int>` values in a
`MinHeap` are a measured workaround (`Heap(by: desc)` 7.7 s vs 1.0 s), and the other gaps
(sorted keyed arrays, `group_by`, `pop_n`, `?? -1`) are reported.

## Checklist failures

- The header says "toys sorted by weight" (line 7); the port never sorts toys, it buckets them
  with `weak.bsearch(w => w > toy.weight)` (line 28) (R1).
- `weak.sort()` in place reuses `Weak` for ranks; reported.

## Language gaps

- All in research/frictions/ioi13p5.md.

## Dream notes

- Names `group_by`, `pop_n`, `partition_point` and `first_true(..) ?? -1`; it elides the reads
  with `...` and puts two statements on one line with `;`, both minor.
