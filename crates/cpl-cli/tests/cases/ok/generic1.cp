// cp: 0.1 4738154+dirty
let n: 1..=10 = read()
let m: 1..=10 = read()
domain R = 1..=n
domain C = 1..=m

fn peel<D>(start: [D]Bool) -> List<D>:
    var up = start
    var gone: [D]Bool = false
    var ready = Set<D>()
    for i in D:
        if start[i]: ready.insert(i)
    var order: List<D> = []
    while let u = ready.min():
        ready.remove(u)
        gone[u] = true
        order.push(u)
        for nb in [u.prev, u.next]:
            if let v = nb:
                if not gone[v]:
                    up[v] = not up[v]
                    if up[v]: ready.insert(v) else: ready.remove(v)
    order

let a: [R]Bool = [R] r => r.idx % 2 == 0
let b: [C]Bool = [C] c => c.idx % 3 != 1
for r in peel(a):
    print(r)
print(peel(b).len)
