// cp: 0.1 329fa7d+dirty
let a: 0..=10 = read()
let b: 0..=10 = read()
fn half(x: Int) -> ?Int:
    if x % 2 == 0:
        return x / 2
    None
let ans: ?Int = do:
    let h = half(a)?
    let g = half(b)?
    if h > g: None else: h + g
print(ans ?? -1)
