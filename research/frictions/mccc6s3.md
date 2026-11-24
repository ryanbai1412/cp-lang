# Friction report: mccc6s3

## Dream

```cp
var right = PresentValues<U>(a)        // deletion-only: nearest present value above/below
for p in Pos:
    right.remove_one(a[p])
    consider(gap, p, None)
    for x in [right.at_or_above(ceil(gap / 2)), right.at_or_below(gap / 2)]: ...
```

## Hand-written skip pointers become `DsuOf`

The v0.2 port kept `pl`/`pr` arrays with recursive `find_l`/`find_r` path compression and
`-1`/`m` sentinels for "nothing there". The rewrite uses two std `DsuOf<U, ?U>`: absent
values join their neighbour above (or below), and the set's value is the nearest present
value. `None` means there is none. A deletion-only "nearest present" structure in std
(`Bitset.next` is O(K/64) per call, which degrades on wide gaps) would replace both.

## Ordering by what is printed

`?Pos` has no order, so the answer key compares `shown(moved)`, the printed move index (`-1`
for none). The statement orders answers by that printed pair.

## The skip lists are hand pointer arrays again

The section above describes the `DsuOf` port; bcc66ff17 and a73c050d6 replaced it with two
hand path-halving arrays (`nxt`, `prv` with `above`/`below`/`drop`) whose ends are sentinels
(`nxt[m] = m`, `prv[0] = 0`, `prv` storing one plus the value). No measurement is recorded for
the swap. Dream line: `var right = PresentValues<U>(a)` with `at_or_above` / `at_or_below`.
General fix: a std deletion-only nearest-present structure over a domain (or `DsuOf` measured
fast enough, with the numbers here). Owner: stdlib. Severity: medium.

## Verification

30 random cases agree with the v0.2 port and 30 with `orig.cpp`. The max case (n = 1e6) runs
in 0.24 s. Corpus tests pass.
