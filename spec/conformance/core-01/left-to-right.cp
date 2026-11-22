// expect: ok, output "-5 123"
// rule: 01.5.4
struct Log:
    var v = 0
    fn put(mut self, d: Int) -> Int:
        self.v = self.v * 10 + d
        d
var g = Log()
let s = g.put(1) - g.put(2) * g.put(3)
print(s, g.v)
