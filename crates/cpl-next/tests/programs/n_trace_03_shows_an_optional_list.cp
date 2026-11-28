// expect: ok, output "YES\nNO"
// A traced run shows a `?List<Int>` as `none` or the list it holds, as `write` writes it.
fn firsts(n: Int) -> ?List<Int> = if n > 0: Some([n, n + 1]) else: None
var o = firsts(0)
print(yes(o == None))
o = firsts(4)
print(yes(o == None))
