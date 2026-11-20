// NOIP 2010 Imprisonment, as it builds today: the adjacency lists and the
// 2-colouring are written out, and the search needs a named domain (K8, K6).
// original: corpus/submissions/5205348.cpp
let n: 1..=2e4 = read()
let m: 1..=1e5 = read()
domain Inmate = 1..=n
let feuds: [m](a: Inmate, b: Inmate, anger: 1..=1e9) = read()

fn peaceful(x: Int) -> Bool:
    var foes: [Inmate]List<Inmate> = []
    for (a, b, anger) in feuds:
        if anger > x:
            foes[a].push(b)
            foes[b].push(a)
    var block: [Inmate]?Bool = none
    for s in Inmate:
        if block[s] != none: continue
        block[s] = false
        var todo = [s]
        while let u = todo.pop():
            for v in foes[u]:
                if let side = block[v]:
                    if side == block[u]!: return false
                else:
                    block[v] = not block[u]!
                    todo.push(v)
    true

domain Anger = 0..=1e9
print(Anger.first_where(x => peaceful(x.label))!)
