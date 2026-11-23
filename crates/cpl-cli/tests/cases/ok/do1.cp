// cp: 0.1 329fa7d+dirty
// `?` is never control flow (R02.4.7): the early exits are `else: return None`.
let a: 0..=10 = read()
let b: 0..=10 = read()
fn half(x: Int) -> ?Int:
    if x % 2 == 0:
        return x / 2
    None
fn both(a: Int, b: Int) -> ?Int:
    let h = half(a) else: return None
    let g = half(b) else: return None
    if h > g: None else: h + g
let ans: ?Int = both(a, b)
print(ans ?? -1)
