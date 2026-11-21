let (n, q): (1..=100, 1..=100) = read()
domain V = 1..=n
let a: [V]Int = read()

var fw = Fenwick<V, Int>(a)
for _ in 0..q:
    let t: 1..=2 = read()
    if t == 1:
        let pos: V = read()
        let delta: Int = read()
        fw.add(pos, delta)
    else:
        let l: V = read()
        let r: V = read()
        print(fw.sum(l..=r))
