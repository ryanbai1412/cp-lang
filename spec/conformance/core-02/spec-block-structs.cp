// expect: ok, output "5 8 7"
// rule: 02.5.4
domain V = 1..=4
struct Edge: u: V, v: V, w: Int

struct Tally:
    var top = 0                     // typed by its default
    var sum = 0
    name: Str = ""
    fn add(mut self, x: Int):
        self.top max= x
        self.sum += x
var t = Tally()
t.add(3)
t.add(5)
print(t.top, t.sum, Edge(1, 2, 7).w)
