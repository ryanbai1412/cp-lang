# Friction report: ioi20p3

## Dream

```cp
@grader fn find_maximum(k: 1..=m, x: [Colour, Ticket]0..=1e9) -> (prize: Int, round: [Colour, Ticket]?0..k)
let taken = [(gain: x[c, Ticket(m - 1 - j)] + x[c, Ticket(k - 1 - j)], c:) for c in Colour for j in 0..k]
    .largest(n * k / 2, by: u => u.gain)
var high = taken.count_by(u => u.c)
var left: [Colour]Deque<Ticket> = Ticket.deque()
var round: [Colour, Ticket]?0..k = None
for r in 0..k:
    for (rank, c) in Colour.sort(by: c => desc(high[c])).items:
        let t = if rank < n / 2: high[c] -= 1; left[c].pop()! else: left[c].pop_front()!
        round[c, t] = r
```

## The grader lives in the solution

`find_maximum` returns the prize and hands the allocation to the grader's
`allocate_tickets`. The port prints both and spells "unused" as `-1` itself; a `@grader`
entry that serialises `?0..k` as the judge's `-1` would remove the output code.

## No partial selection

The best `n*k/2` upgrades are a full `sorted(by: desc(..))` then `take`. A std
`largest(k, by:)` (nth_element plus a partial sort) is the spelling and is O(nk) instead of
O(nk log nk).

## Counting by key and a deque of a domain

`high` is a zeroed table plus a counting loop (`count_by` would say it), and each colour's
ticket deque is `Deque.of(Ticket.collect())`, materialising the domain as a list first.

## Rounds are the same for different `round` types

`round` is `?Int`: the round index has no `0..k` type because `r` comes from `0..k` and a
range over a runtime bound does not give its elements that type.

## Verification

The allocation is not unique, so `checker.py` checks the prize against the answer's and that
the allocation is valid and pays it. 30 random cases agree with the v0.2 port and 30 with
the accepted C++ (`orig.cpp`, the AC submission plus a stdin `main` standing in for the
grader), and the max case (n = m = k = 1500) passes the checker in 0.45 s.
