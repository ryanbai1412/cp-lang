// IOI 2020 Day 1 P2, Connecting Supertrees
// original: corpus/submissions/4724578.cpp
// FRICTION: `build` is declared at file scope but its `V` only exists inside `construct`
@grader fn build(b: [V, V]Bool)

@grader
fn construct(p: [V, V]0..=3) -> Bool:
    domain V                       // inferred from the shape of p
    // Towers with p == 1 form trees; hang each tree off its first tower as a star.
    var root: [V]?V = none
    var roots: List<V> = []
    for i in V if root[i] == none:
        roots.push(i)
        for j in V if root[j] == none and p[i, j] == 1: root[j] = i

    // Trees whose roots have p == 2 sit on one cycle; ring[v] names that cycle.
    var ring: [V]?V = none
    var bridge: [V, V]Bool = false
    for i in roots if ring[i] == none:
        let cycle = [j for j in roots if ring[j] == none and (j == i or p[i, j] == 2)]
        for j in cycle: ring[j] = i
        if cycle.len == 2: return false
        if cycle.len > 2:
            for k in 0..cycle.len:
                let (a, b) = (cycle[k], cycle[(k + 1) % cycle.len])
                bridge[a, b] = true
                bridge[b, a] = true
    for v in V if v != root[v]!:
        bridge[v, root[v]!] = true
        bridge[root[v]!, v] = true

    // the layout determines every path count, so p must agree with it exactly
    for i in V:
        for j in V:
            let (a, b) = (root[i]!, root[j]!)
            let want = if a == b: 1 elif ring[a] == ring[b]: 2 else: 0
            if p[i, j] != want: return false
    build(bridge)
    return true
