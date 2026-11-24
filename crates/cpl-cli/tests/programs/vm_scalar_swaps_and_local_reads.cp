// Swaps of plain values read them rather than take them, local list, array
// and map cells read as scalars, and a set in a map entry takes inserts: all
// on the VM, in the interpreter as in C++.
struct P:
    a: Int
    b: Int

fn go(n: Int) -> Int:
    var ps = [P(a: 1, b: 2), P(a: 3, b: 4)]
    swap(mut ps[0], mut ps[1])
    var xs = [n, n + 2, n + 4]
    swap(mut xs[0], mut xs[2])
    var g: [0..2, 0..2]Int = 0
    g[0, 1] = n
    swap(mut g[0, 1], mut g[1, 0])
    var m: Map<Int, Int> = [:]
    m[n] = 7
    var sets: Map<Int, Set<Int>> = [:]
    sets[1] = Set()
    sets[1].insert(n)
    sets[1].insert(n + 1)
    sets[1].remove(n)
    print(ps[0].a, ps[1].b, xs[0], xs[2], g[1, 0], g[0, 1], m[n], sets[1].len)
    xs[1] + g[1, 0] + m[n]

print(go(5))
