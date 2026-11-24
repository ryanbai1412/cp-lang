# Friction report: noi05p1

## Dream

```cp
for line in Cell.lines(way: Dir4):                 // L3
    for run in line.split(p => hall[p] == 'x'):    // L16
        next[run] = run.sliding(max, width: seconds + 1, of: far - place) + place
```

## Hand monotone deque becomes `Window(max)`

The v0.2 port kept a `List` plus a moving `head` index as its monotone queue. The rewrite
uses std `Window(max)` for the best `slide - place` in reach and a `Deque` of
`(value, place)` to know which value leaves when. A start that falls out of reach is
`pop_front_while`. The direction table is a `match`; `way` stays `1..=4` (N S W E) because
the input numbers it.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. Corpus tests pass
8/8. The max generator case takes 0.45 s against a 400 ms limit. The second structure costs
about 2x over a single hand deque, and `Deque` has no `pop_back_while` to fold the two
together.
