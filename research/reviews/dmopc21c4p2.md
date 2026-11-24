# dmopc21c4p2

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the dream and the port.

Sizes that divide no smaller present size each start a group, and each group covers the multiples of its size: a sieve over `Size`.

## Checklist failures

- `for m in (v.label..=Size.last!.label).step(v.label): covered[Size(m)] = true` walks multiples through labels and back into the domain.
- `present` is filled by a loop; `Bitset.of(size)` or a comprehension would make it a `let`.
