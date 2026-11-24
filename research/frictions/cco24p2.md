# cco24p2 frictions

## Dream

```cp
elif a.is_permutation(Flavour):
...
elif n <= 10:
    let fewest = matchings(Pizza, Person, where: (p, q) => a[p] == b[q])
        .map(m => decreasing_piles(Pizza.map(p => m[p])))
        .min(by: piles => piles.count, prune: piles => piles.count)!
    print_stacks(fewest.count, p => fewest.pile_of(p), q => fewest.pile_of(fewest.matched(q)))
```

## Domain as a permutation target

`a.is_permutation(Flavour)` fails inside `std.seq` ("a `for` can't walk over a `Flavour`"),
so the port collects the domain first: `a.is_permutation(Flavour.collect())`. A domain is a
sequence everywhere else; the std generic should accept it directly.

## Backtracking with manual undo

```cp
let old = if i < tops.len: Some(tops[i]) else: None
if i == tops.len: tops.push(q)
else: tops[i] = q
...
search(p + 1)
if let top = old: tops[i] = top
else: tops.pop()
```

Dream: `matchings(Pizza, Person, where: ...)` with a `prune:` hook, and `decreasing_piles(seq)`.
The port keeps the search state in top-level `var`s and undoes each step by hand, which is the
C++ shape and easy to get wrong (forgetting `taken[q] = false` compiles). Severity: medium. Fix
(stdlib): a pruning `matchings` / `permutations` generator steered like `step.prune()`, and a
patience-pile decomposition that returns each element's pile; together they remove the state
and the undo. `cpl run` on the dream line today: ``error[E0201]: no `matchings` in scope, and not in
the stdlib (yet)`` (help points at `Matching` in `std.flow`, a graph matching, not a generator).

## Max-input memory 1.14x the C++

On `gen.py --max` (N = 1e6, a 13.8 MB input) the port peaks at 42.7 MB against the accepted
C++'s 37.6 MB, the same as before the search was added; time is at or under the C++'s
(0.16 s against 0.23 s, best of 5). The emitted runtime reads all of stdin into one growing
buffer (`in.resize(have + (1 << 16))`) where `cin` streams, so RSS carries the whole input.
Severity: low. Fix (codegen): read in fixed-size chunks and parse as it goes.
