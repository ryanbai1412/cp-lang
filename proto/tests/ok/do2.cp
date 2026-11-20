// cp: 0.1 329fa7d+dirty
let a: 0..=10 = read()
let b: 0..=10 = read()
fn half(x: Int) -> ?Int:
    if x % 2 == 0: return x / 2 else: return none

fn f(x: Int, y: Int) -> Int:
    let r: ?Int = do:
        let h = half(x)?
        if h == 0:
            return 100
        let g = half(y)?
        if h > g: none else: h + g
    r ?? -1

print(f(a, b))
var total = 0
for i in 0..=a:
    let w: ?Int = do:
        if i == 5:
            break
        let h = half(i)?
        h
    total += w ?? 0
print(total)
