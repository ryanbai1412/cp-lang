// expect: ok, output "2"
// rule: 05.7.10
var kept = [0]
fn f(xs: List<Int>):
    kept = xs.clone()
f([1, 2])
print(kept.len)
