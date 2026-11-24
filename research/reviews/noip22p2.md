# noip22p2

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port. With no dream.cp the port scores at most 3 (README); against the checklist alone it reads as a 4.

Cold review: moves are a Move enum instead of integer tuples; card times, cards and stacks are separate domains; twin and home are optionals; the three stack pools are Bitset<Stack>. Bitset<Stack> iterated directly yields Int, so the longest-staying single stack is found through .each().

## Checklist failures

- `empty`, `single` and `ripe` are three `Bitset<Stack>`s kept as a partition by `refresh`, which removes a stack from all three and re-adds it to one.
- `leaves(t) = twin[t]!` unwraps a twin that every dealt card has, but the type does not say so; `twin` could be built as a `[Time]Time` once the deck is read.
- `unmatched[deck[t]] = None` and `= t` pair cards with an optional table where a `Map` or a pairing pass over the deck would read as "pair the two equal cards".

## Gaps that need the language

- Three Bitsets as a hand-kept partition: research/frictions/noip22p2.md.
