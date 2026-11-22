// expect: ok, output "1"
// rule: 05.7.9
fn keep(mut out: List<Str>, s: Str): out.push(s.clone())
var o: List<Str> = []
keep(mut o, "a")
print(o.len)
