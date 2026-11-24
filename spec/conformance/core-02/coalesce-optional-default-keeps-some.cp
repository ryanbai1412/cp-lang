// expect: ok, output "YES YES YES YES\n1"
// rule: 02.4.5
// An optional default keeps the left optional whole, for a non-scalar payload too.
fn s() -> ?List<Int> = Some([1])
fn t() -> ?List<Int> = None
let x = s() ?? t()
print(yes(x == Some([1])), yes(x! == [1]), yes((Some([1]) ?? None) == Some([1])), yes((t() ?? s()) == x))
print(x!)
