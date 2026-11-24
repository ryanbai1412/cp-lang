# Friction report: noi01p5

## Dream

```cp
let (left, right) = T.split(n / 2)
print(count_pairs(sums(left), sums(right), (x, y) => x + y == 0))
```

## Sorted halves instead of a counts map

The v0.2 port put the 3.4e6 first-half sums into a `counts` map and peaked at 344 MB on the
max case. The rewrite sorts the first half and walks the sorted second half with
`equal_range`. That peaks at 88 MB and runs in 0.72 s; the accepted C++ takes 36 MB and 0.29 s.
The halves come from `partition` instead of two filtered comprehensions, and the unused
`pow` helper is gone.

## Verification

`gen.py` is new (the max case is n = 6, M = 150, every p = 3). 30 random cases agree with the
v0.2 port and 30 with the accepted C++. Corpus tests pass 6/6, including the max case,
which was MLE before.

## Counter port, measured

The port now counts the first half's sums in a `Counter<Sum, 0..=3_375_000>` as they are
generated and looks each second-half sum up as it is made; the sorted-halves version above is
gone. On the `gen.py --max` case the built port takes 0.29 s and 42 MB (`/usr/bin/time`),
inside the 1000 ms / 64 MB limit.

## No std enumeration of the sums over a product of lists

`count_left` and `matches` (port lines 23-34) are the same recursive walk, with an `at`
cursor, over one value per term:

```cp
fn count_left(at: Int, s: Int):
    if at == left.len:
        low.add(s, 1)
        return
    for v in values[left[at]]: count_left(at + 1, s + v)
```

The dream line is `sums(left)`; std has no such function:

```
error[E0201]: no `sums` in scope
  --> /tmp/s.cp:2:7
  |
2 | print(sums(vals).len)
  |       ^^^^
```

General fix: a std generator over a list of lists that yields every sum (or every fold under a
given operation) taking one element from each, lazily, so meet-in-the-middle halves are one
call each. Owner: stdlib.

## Tuned twin retired (2026-11-24)

`perf.cp`'s code was byte-identical to `port.cp` (only its comments differed): the port already adds the first half's sums to the counter as they are made, so the twin bought nothing and is deleted. On `gen.py --max` (best of 7) the port runs 0.274 s / 40.5 MB against 0.43 s / 34.2 MB (0.64x time, 1.18x memory); the memory gap is the half-sum counter itself, which the twin did not change, and stays below as a friction.
