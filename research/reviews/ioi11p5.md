# ioi11p5

Score: pending cold rescore (port = dream: replaced hand `Bucket.first_ge`/`first_gt` with `xs.first_ge`/`xs.first_gt`, removed `span` threading by reading top-level `l`, dropped `let _ =` on `xs.remove`; 0 lints, `22,027 B`, verified on 50 ASan+UBSan cases vs brute force)
Dream: pending cold rescore (port = dream 1.000x, 0 errors)

Baseline review, cold, on main 96359635. No editorial header; lines 1-3 describe prototype deltas, not the algorithm. The body is the C++ sqrt decomposition: three parallel `List<List<Int>>` bucket arrays (lines 18-20), a hand insertion sort and shift-delete (lines 62-82), and a `has_cover` flag fold (lines 126-137). No dream exists, so the score is capped at 3.

## Checklist failures

- No header for the idea: buckets of sorted positions, each storing the jumps to leave the bucket (lines 1-3).
- Hand-rolled binary search `first_gt_idx` (lines 84-93); `bsearch_first` (`stdlib/seq.cp:73`) exists.
- Integer sqrt by `while B * B < n: B += 1` (lines 12-16), and the clamp `B < 2` is unexplained.
- Parallel arrays `bxs`/`bend`/`bcnt` (lines 18-20) instead of one bucket struct.
- Insertion and deletion written as hand `while` loops (lines 64-81).
- Two linear bucket scans with `break` (lines 105-119) that differ only in the predicate.
- Flag `has_cover` plus a `cover = 0` sentinel (lines 127-136) where `?Int` works.

## Rewrites owed with today's language

- Declare `struct Bucket(xs: List<Int>, jumps: [..](end: Int, count: Int))` and rebuild it as a value with one comprehension.
- Use `bsearch_first` for `first_gt_idx` and for the two bucket lookups.
- Fold the answer over buckets with `cover: ?Int`.

## Gaps that need the language

- None needed for a 4: everything above is today's language. The port has no friction report.

## Cold review (HEAD)

Dream (new) 3/5, port 3/5 (port = dream), cold-reviewed on current main; sent back. It checks with zero errors, and the header derives the bucket size, the per-bucket jumps and the query cost.
- Owed: `Bucket.first_ge`/`first_gt` (lines 36-43) re-implement binary search over `xs`; sorted lists already have `xs.first_ge(x)` and `xs.first_gt(x)`.
- Owed: the camera length `l` is fixed for the whole run, yet `span` is threaded through `rebuild`, `remove`, `insert` and `make_bucket` (lines 23, 45, 49, 53, 72, 82-83).
- Minor: `let _ = self.xs.remove(...)` (line 46); bucket ids are raw `Int`s, with `?? (buckets.len - 1)` as the fallback (line 60).
- No friction report is needed: everything owed exists.
