// expect: ok, output "YES"
// jump_while, dist and after on long tails agree with walking `nx` step by step.
fn first_at<T: Eq>(xs: List<T>, p: T -> Bool) -> ?Int:
    for i in 0..xs.len:
        if p(xs[i]): return Some(i)
    None
var ok = true
for trial in 0..30:
    domain V = 0..60
    let nx: [V]V = [V] v => if v.ord > 0 and rand(0, 9) > 0: V(rand(0, v.ord - 1)) else: V(rand(0, 59))
    let f = Functional(nx)
    let mark: [V]Bool = [V] _ => rand(0, 3) == 0
    for v in V:
        var walk = [v]
        for _ in 0..130: walk.push(nx[walk.last!])
        for s in [0, 1, 7, 59, 130]:
            if f.after(v, s) != walk[s]: ok = false
        let t = V(rand(0, 59))
        if f.dist(v, t) != first_at(walk, u => u == t): ok = false
        let cut = first_at(walk, u => mark[u] or f.on_cycle(u))!
        let d = f.depth(walk[cut])
        if f.jump_while(v, u => f.depth(u) <= d) != Some(walk[cut]): ok = false
        if f.jump_while(v, u => false) != None: ok = false
print(if ok: "YES" else: "NO")
