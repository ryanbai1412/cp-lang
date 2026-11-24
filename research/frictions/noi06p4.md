# Friction report: noi06p4

## Dream

```cp
domain Node = Station + User + End     // disjoint-union domain (spec R02.9.6)
net.add(Node(u), Node(users[u].a), None)
```

## Sum domains don't construct

Spec R02.9.6 names `domain Node = In + Out`, but `Node(s)` for an `s: Station` is still
E0302 (expected `Int`). The port keeps one numbered `Node` domain behind two small `station`
and `group` maps, plus a `source` and `sink` at the end.

## No sentinel capacity

The v0.2 port used capacity `1e9` on the group-to-station arcs ("never cut"). `FlowGraph.add`
takes `?Int`, and `None` means uncapped, so the rewrite passes `None`.

## Verification

30 random cases agree with the v0.2 port and 30 with the accepted C++. A first draft mapped
stations with `Int(s)` (the 1-based label), which collided with the groups. Stress caught it
and it now uses `s.ord`. Corpus tests pass 8/8.
