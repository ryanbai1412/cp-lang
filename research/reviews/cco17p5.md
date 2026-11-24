# cco17p5

Score: 4/5
Dream: 4/5

The greedy is the header: walk k from n down, push the people with need k into a min-heap, and pay the cheapest while too few others remain. Four lines of logic.

## Checklist failures

None beyond the reported gap.

## Owed rewrites

- None. std's `buckets` (`[D]K -> [K]List<D>`) buckets indices, not payloads, and the friction's `buckets(over:, by:, of:)` fails under 0.2 with `error[E0310]: buckets has no parameter named over`, so the declaration and push loop stay.

## Language gaps

- Bucketing by a key spelled as a declaration and a push loop: research/frictions/cco17p5.md.

## Dream notes

- The dream is the port with the bucketing named; no header.
