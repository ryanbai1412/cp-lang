# cco24p4 frictions

## Dream

```cp
for who in Agent:
    for start in Room:
        print(climbs.items.scan(start, (at, (minute, climb)) => if climb and minute % 2 == who.ord: tree.parent(at) ?? at else: at))
```

## Running state as a row

Each output row is the agent's room after every minute: a left scan over the schedule. The
port keeps a `var at` and pushes to a row, because `scan` with a seed and a tuple element reads
worse than the loop today.
