# ioi09p5: Garage

## Dream

```cp
enum Event = Arrive(Car) | Leave(Car)  // read from a signed integer
for e in read<[2 * m]Event>():
    match e:
        Arrive(c) => waiting.push(c)
        Leave(c) => free.push(parked[c]!)
    while let (car, spot) = (waiting.pop_front(), free.pop()):
        ...
```

## A signed event can't decode into a tagged enum
- **Where:** `event: -m..=m` and the `event > 0` test; the sign is the tag.
- **General fix:** a `read` mapping for sign-tagged inputs (shared with the
  command-stream frictions).
- **Severity:** low.

## Two pops that both must succeed are a loop guard plus two unwraps
- **Where:** `while waiting.len > 0 and free.len > 0:` then `pop_front()!`,
  `pop()!`; a `while let` over a tuple of optionals would say it once.
- **Severity:** low.
