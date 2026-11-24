# ioi09p2

Score: 4/5
Dream: 4/5

## Cold review (HEAD)

Port 4/5, dream 4/5, from the friction's `## Dream`. The header (lines 6-10) derives the rate-setter argument and the heap sweep, and lines 18-43 are it: exact `Ratio` rates, a `MaxHeap` popping the largest Q while over budget, and the best `Plan` by size then bill.
- The running sum beside the `MaxHeap` (lines 30-36) is in the friction (no k-smallest-summed structure).
- The friction's `Ratio.new(x, 1)` entry is stale: line 18 builds `Ratio.new(s, q)` once and the bill comparison (lines 24-28) cross-multiplies integers.
