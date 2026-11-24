# ioi12p5 review

Score: 3/5
Dream: none (no dream.cp; the ideal is in the friction report)

Scored cold against the checklist and the port.

Cold review: rethought the bookkeeping. The old port kept `stay: [Colour]Int` with -1 for "not
on the scaffold", indexed advice bits by raw Int, and took Belady's victim with `on.max()` then a
second `on.remove(on.max()!)`. Now stays are their own `Stay` domain (`opened(t)`, `initial(c)`
returns `?Stay`), `stay` is `[Colour]?Stay`, the victim is one `on.pop_last()`, and the assistant
starts from a filtered comprehension of the initial spare colours. Times keep n as "never
requested again", which is what orders the Belady set. Adds tests.toml (seeded sizes plus --max).

## Checklist failures

- `next_use: [Step]Int = n` and `upcoming: [Colour]Int = n` use the time `n` as "never requested again" (R5). It works only because the Belady set orders by it; a `?Step` with `None` sorting last would say so.
- `on` is a `Set<(Int, Colour)>` keyed by a raw time instead of a `Step`.
- `opened(t) = Stay.nth(k + t.ord)` and `initial(c)` build a stay from ordinal arithmetic, splitting one `Stay` domain into two ranges by hand.
- `print([Int(b) for b in needed])` converts a `Bool` array to digits by hand.

## Gaps that need the language

- Two programs run as one stdin program, and "never again" as time n: research/frictions/ioi12p5.md.
