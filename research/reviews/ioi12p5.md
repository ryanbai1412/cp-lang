# ioi12p5 review

Score: pending
Dream: pending

Cold review: rethought the bookkeeping. The old port kept `stay: [Colour]Int` with -1 for "not
on the scaffold", indexed advice bits by raw Int, and took Belady's victim with `on.max()` then a
second `on.remove(on.max()!)`. Now stays are their own `Stay` domain (`opened(t)`, `initial(c)`
returns `?Stay`), `stay` is `[Colour]?Stay`, the victim is one `on.pop_last()`, and the assistant
starts from a filtered comprehension of the initial spare colours. Times keep n as "never
requested again", which is what orders the Belady set. Adds tests.toml (seeded sizes plus --max).
