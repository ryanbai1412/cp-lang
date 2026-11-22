// expect: ok, output "2"
// rule: 02.6.4
struct C:
    var n = 0
    fn bump(mut self): self.n += 1
    fn get(self) -> Int = self.n
var c = C()
c.bump()
c.bump()
print(c.get())
