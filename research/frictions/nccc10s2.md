# Friction report: nccc10s2

## Dream

```cp
let ballots: [n]Perm<L> = read()
let ahead = [L, L] (x, y) => ballots.all(b => b.pos(x) < b.pos(y))
print(Graph.dag(ahead).longest_path_len())
```

## Ballots read as `Str`

Each ballot is a permutation of the first k letters but is read as `Str`, with `L(x)` per
character. Reading a `Perm<L>` would type it and check it is a permutation.

## Longest chain by `@memo`

The longest path in the "ahead on every ballot" DAG is a memoised recursion. A std
longest-path over a DAG given as a `[L, L]Bool` relation would name it.

## Already at bar otherwise

Apart from the tuple read and dropping the stale `cp:` line and the WISH comment, the v0.2
port was already at bar. 30 random cases agree with the v0.2 port and 30 with the accepted
C++. The generator's size-0 case is fixed. Corpus tests pass.
