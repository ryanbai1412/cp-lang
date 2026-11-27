// expect: ok, output "12 20 5"
// A compound assignment to a field computes into a temporary the optimizer forwards into the
// field's assignment, so the call it holds still runs before the field is written.
struct C:
    var n: Int
    var hits: Int
    fn bump(mut self) -> Int:
        n += 10
        hits += 1
        return n
var c = C(1, 0)
c.n += c.bump()
let a = c.n
c.n = c.bump() - c.hits
c.hits += 3
print(a, c.n, c.hits)
