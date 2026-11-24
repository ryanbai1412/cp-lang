# Friction Report: cco09p4

The v0.2 port was already a three-way quicksort of caps against bottles over two domains, but it
drew its pivot with the removed `rand(lo, hi)` and tested `caps.len == 0`. Now it draws from a
seeded `Rng` with `rng.pick(bottles)!` and tests `caps.is_empty`. The package gains a gen.py of
hidden size lists, an interactor.py that answers `0 c b` and checks every `1 c b` report and
the 5e5-query budget, and a tests.toml.

## `group` returns a map, so the two sides need `get(..) ?? []`
- **Where:** `pair_up(by_fit.get(-1) ?? [], below)` and `pair_up(by_fit.get(1) ?? [], above)`.
- **Ideal:** a three-way partition, `let (small, fits, large) = caps.partition3(by: c => fit(c, pivot))`.

## The --max case is left out
At n = 1e4 the round trips through the Python interactor take about 7 s for both the port and
the accepted C++ (334030 and 330885 queries), so tests.toml stops at size 150 (n <= 452).

## Dream
```cp
fn pair_up(caps: List<Cap>, bottles: List<Bottle>):
    if caps.is_empty: return
    let pivot = rng.pick(bottles)!
    let (small, fits, large) = caps.partition3(by: c => fit(c, pivot))
    let cap = fits[0]
    print(1, cap, pivot)
    let (below, above) = bottles.without(pivot).partition(b => fit(cap, b) > 0)
    pair_up(small, below)
    pair_up(large, above)
```
