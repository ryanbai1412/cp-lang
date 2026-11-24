let n: Int = read()
domain V = 0..n
domain D = 0..16 * n
domain T = 0..4
var s = 0
for i in 0..8 * n:
    let w0 = Window(+) // timed: window.Window.new
    let t0 = track(0, (mut s, x) => s += x, (mut s, x) => s -= x) // timed: window.track
    let u0 = undoing(Dsu<T>(rollback: true), (mut d, e: (T, T)) => d.unite(e.0, e.1)) // timed: window.undoing
    let sw = Sweep<D, Counter<D, Int>>(Counter<D, Int>()) // timed: window.Sweep.new
    s += w0.len + t0.state + sw.range.len
var w = Window(min)
var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x)
for i in 0..8 * n:
    w.push(i * 7919 % n) // timed: window.Window.push
    t.push(i) // timed: window.Tracked.push
    if i >= 5:
        w.pop((i - 5) * 7919 % n) // timed: window.Window.pop
        t.pop(i - 5) // timed: window.Tracked.pop
    s += w.all ?? 0 // timed: window.Window.all
    s += w.len // timed: window.Window.len
    s += t.state // timed: window.Tracked.state
var u = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1))
for i in 0..4 * n:
    u.push((V(i * 7 % n), V(i * 13 % n))) // timed: window.Undoing.push
    if i % 2 == 1:
        u.pop((V(0), V(0))) // timed: window.Undoing.pop
    if u.state.same(V(0), V(i % n)): s += 1 // timed: window.Undoing.state
var sw = Sweep<D, Counter<D, Int>>(Counter<D, Int>())
let step = n / 64
for i in 0..512:
    let lo = i * step
    sw.move(D(lo)..D(lo + 4 * step)) // timed: window.Sweep.move
for i in 0..8 * n:
    s += sw.range.len // timed: window.Sweep.range
print(s)
