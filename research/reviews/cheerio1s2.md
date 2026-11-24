# cheerio1s2

Score: 4/5
Dream: 3/5

Cold review against the friction's `## Dream`. The header (lines 5-7) argues why the N cheapest rooms leave no gap; the heap loop (lines 14-20) is that argument.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- `MinHeap` from a sequence (the push loop, line 15): in the friction.
- A fused pop-then-push (lines 18-20): in the friction.

## Dream notes

- `sum(next.pop_push(w => w + b) for _ in 0..n)` hides the heap mutation inside a fold (R7), which caps the dream at 3. Keep the fused call as a statement in the loop.
