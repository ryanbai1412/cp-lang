domain V = 0..6
var back: [V]?(from: V, via: (to: V, w: Int, up: Bool)) = None
for v in V:
    if v.ord % 3 != 1:
        back[v] = Some((from: v, via: (to: V(5 - v.ord), w: v.ord * 10 - 7, up: v.ord % 2 == 0)))
var total = 0
for v in V:
    if let b = back[v]:
        total += b.via.w + b.from.ord
        if b.via.up: total += 100
print(total)
back[V(0)] = None
print(yes(back[V(0)] == None), yes(back[V(2)] == Some((from: V(2), via: (to: V(3), w: 13, up: true)))))
var seen: List<?(Int, (Int, Char))> = []
seen.push(None)
seen.push(Some((2, (3, 'x'))))
seen.push(Some((1, (9, 'y'))))
seen.reverse()
print(seen.len, yes(seen[2] == None), yes(seen[0] == Some((1, (9, 'y')))))
