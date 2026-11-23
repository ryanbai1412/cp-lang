var calls = 0
fn tick() -> ():
    calls += 1
let n: Int = read()
var xs: List<(Int, ())> = []
for i in 0..n: xs.push((i, ()))
let f = (a: (), b: ()) => a
var us: List<()> = []
us.push(f((), ()))
us.push(tick())
us[0] = tick()
let o: ?(Int, ()) = Some((n, ()))
print(xs.len, us.len, calls, o.is_some)
