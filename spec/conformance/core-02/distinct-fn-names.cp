// expect: ok, output "1 0"
// rule: 02.11.3
fn f_int(x: Int) -> Int = x
fn f_str(x: Str) -> Int = 0
print(f_int(1), f_str("a"))
