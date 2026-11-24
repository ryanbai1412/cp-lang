# nccc10s3

Score: 4/5
Dream: 4/5

Cold review: the suffix array's rank range for t and a `WaveletMatrix` k-th smallest start (lines 17-22), the header's argument. The bound check before `kth` is redundant because `kth` already returns an option.

## Checklist failures

- R4: `if k <= ranks.len: starts.kth(ranks, k - 1)! + 1 else: -1` (line 22) checks the bound and then unwraps `kth`, which returns `?Int` (`std/segtree.cp` line 1237) and is `None` past the range.
- R8: `for _ in 0..q` (line 19) where `repeat q:` is the spelling.

## Owed rewrites

- `print(if let i = starts.kth(ranks, k - 1): i + 1 else: -1)` and `repeat q:`.

## Language gaps

- The `k` bound read inside the loop: in research/frictions/nccc10s3.md. Its `kth` traps section is corrected (std `kth` returns `?Int`).

## Dream notes

- None.
