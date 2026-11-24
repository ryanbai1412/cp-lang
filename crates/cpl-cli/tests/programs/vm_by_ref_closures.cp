// A closure that captures a `var` by reference is made on the vm: it sees
// later writes, and its writes reach the variable, for a scalar local, a
// list, a `mut` param, and the field of a `mut self` held as its one Int.
struct Acc:
    total: Int
    fn add_all(mut self, xs: List<Int>) -> Int:
        let f = (x: Int) => total += x
        for x in xs: f(x)
        let seen = () => total
        total *= 2
        seen()

fn bump(mut x: Int, by: Int) -> Int:
    let get = () => x
    x += by
    get() + x

let n: Int = read()
var count = 0
var hits: List<Int> = []
let note = (k: Int) => if k % 3 == 0:
    count += 1
    hits.push(k)
for k in 1..=n: note(k)
let peek = () => count * 100 + hits.len
count += 1
print(count, hits, peek())
var c = 2
print(bump(mut c, n), c)
var acc = Acc(0)
print(acc.add_all([1, 2, n]), acc.total)
