# Friction report: mccc4j5

## Dream

```cp
let tasks: [V](time: 1..=1e4, deps: List<V>) = read()
let g = Graph.directed([(d, u) for u in V for d in tasks[u].deps])
print(g.kahn_fold(u => tasks[u].time, (f, u) => f + tasks[u].time, max).values.max() ?? 0)
```

## Longest path in the acyclic part

std `topo` and `dag_fold` return `None` on any cycle, but here the events on or after a cycle
are just skipped. The port keeps a Kahn walk driven by a min-heap of finish times. A
`dag_fold` variant that folds the part Kahn's algorithm reaches would make it one call.

## Already at bar

The v0.2 port already had typed reads, a `V` domain, std `MinHeap` and no sentinels. The
re-port only adds the `original:` line. 30 random cases agree with the v0.2 port and 30
with the accepted C++. The max case runs in 0.25 s (limit 1.5 s). Corpus tests pass.
