let (n, t) = read<(1..=2e5, 1..=1e9)>()
let k = read<[n]1..=1e9>()
fn made(time: Int) -> Int = k.sum(of: x => min(time / x, t))
print((1..=k.min()! * t).first_true(time => made(time) >= t)!)
