// expect: ok, output "1"
// rule: 08.4.2
fn f(e: (w: Int)) -> Int = e.w
print(f((w: 1)))
