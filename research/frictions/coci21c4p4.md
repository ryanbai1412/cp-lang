# Friction report: coci21c4p4

Rethought. The v0.2 port built a `[Town]List<Town>` adjacency by hand and copied each road length into an `up` side array. Its header carried an "as it builds today" note. The v0.3 port roots `Graph.undirected(roads)` and reads each length from `tree.up_arc(v)`'s `w`, so no side array remains. Recomputing the parks for the best radius matches dream.cp.

## Gap

dream.cp folds each vertex's children with `t.up(leaf: .., (v, kids) => ..)`, an arc-aware bottom-up fold. The port keeps the explicit `tree.order.rev` sweep with `far` and `near` arrays, because `Rooted.up` combines children through one monoid op and has no hook to push a park as it goes.

## Measurements

gen.py --max: port 0.28 s, v0.2 port 0.46 s, same radius and output. The accepted C++ recurses and dies (signal 11) under an 8 MB stack. With an unlimited stack it takes 20.8 s on this machine and gives the same radius; checker.py accepts the port's parks.
