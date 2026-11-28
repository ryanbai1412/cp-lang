// expect: ok, output "0\n3"
// A traced run shows a `?Int` as `none` or the `Int` it holds, through std's trace fns.
fn half(n: Int) -> ?Int = if n % 2 == 0: Some(n / 2) else: None
var o = half(3)
print(o ?? 0)
o = half(6)
print(o ?? 0)
