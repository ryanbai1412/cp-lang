// The sizes plan/80-validation.md 8.2 promises for core types.
// `cpl-verify layout` measures each `//! sizeof` line in the emitted C++.
let n: 1..=200000 = read()
domain V = 1..=n
struct Edge:
    u: V
    v: V
    w: 1..=1000000000
var pos: V = V(1)                           //! sizeof pos == 4
var maybe_pos: ?V = none                    //! sizeof maybe_pos == 4
var maybe_int: ?Int = none                  //! sizeof maybe_int == 8
var maybe_small: ?0..=1000000000 = none     //! sizeof maybe_small == 4
var dist: [V]Int = 0                        //! sizeof dist[] == 8
var parent: [V]?V = none                    //! sizeof parent[] == 4
var limit: [V]0..=1000000000 = 0            //! sizeof limit[] == 4
var edges: List<Edge> = []                  //! sizeof edges[] == 12
var pairs: List<(V, V)> = []                //! sizeof pairs[] == 8
print(pos.idx, dist[pos], limit[pos], edges.len, pairs.len)
