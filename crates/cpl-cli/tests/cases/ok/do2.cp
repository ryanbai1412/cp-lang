// cp: 0.1 329fa7d+dirty
// Early exits from a computation: `else: return` for a missing value, `break` out of the loop.
let a: 0..=10 = read()
let b: 0..=10 = read()
fn half(x: Int) -> ?Int:
    if x % 2 == 0: return x / 2 else: return None

fn pair(x: Int, y: Int) -> ?Int:
    let h = half(x) else: return None
    if h == 0: return 100
    let g = half(y) else: return None
    if h > g: None else: h + g

fn f(x: Int, y: Int) -> Int = pair(x, y) ?? -1

print(f(a, b))
var total = 0
for i in 0..=a:
    if i == 5: break
    total += half(i) ?? 0
print(total)
