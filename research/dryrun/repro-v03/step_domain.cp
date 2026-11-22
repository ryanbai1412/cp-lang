// `p.step(d)` on a cell popped from a `Deque<(R, C)>`: `cpl check` accepts; `cpl run` sees `Int`s.
domain R = 0..2
domain C = 0..2
var seen: [R, C]Bool = false
var queue = Deque<(R, C)>.new()
queue.push((R(0), C(0)))
while let p = queue.pop_front():
    for d in Dir4.all():
        if let q = p.step(d):
            if not seen[q]:
                seen[q] = true
                queue.push(q)
print(seen[(R(1), C(1))])
