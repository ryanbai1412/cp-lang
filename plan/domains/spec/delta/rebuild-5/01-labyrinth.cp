// CSES 1193 Labyrinth: shortest path from A to B in a grid of '.', '#', 'A', 'B'.
// Input: n m, then n rows of m chars. Output: YES, the length and the moves (LRUD), or NO.
let (n, m) = read<(1..=1000, 1..=1000)>()
domain R = 0..n
domain C = 0..m
let g = read<[R, C]Char>()
let a = (R, C).find(p => g[p] == 'A')!
let b = (R, C).find(p => g[p] == 'B')!

fn moves(p: (R, C)) -> yields (to: (R, C), ch: Char):
    for ch in "URDL":
        if let q = p.step(Dir4.from("URDL", ch)!), g[q] != '#':
            yield (to: q, ch:)

let route = bfs(moves, from: [a], until: b).arcs(b) else:
    print("NO")
    return
print("YES")
print(route.len)
print(Str(x.ch for x in route))
