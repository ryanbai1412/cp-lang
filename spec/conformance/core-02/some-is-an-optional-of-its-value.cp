// expect: ok, output "3 -1 2 4"
// rule: 02.4.2
// `Some(x)` is a present `?T` of `x`'s type, so a branch, element or return built from it is typed `?T`.
fn half(x: Int) -> ?Int = if x % 2 == 0: Some(x / 2) else: None
let xs = [Some(3), None]
let p = if xs.len > 1: Some(xs.len) else: None
let q = match p:
    Some(n): Some(n * 2)
    None: None
print(xs[0]!, half(3) ?? -1, p!, q!)
