// expect: ok, output "2 2"
// rule: 06.1.2 05.3.2
struct Acc:
    var t: Int
    fn add(mut self, x: Int): self.t += x
fn grow(mut xs: List<Int>): xs.push(1)
var a = List<Int>()
grow(mut a)
a.push(2)
var s = Acc(t: 0)
s.add(2)
print(a.len, s.t)
