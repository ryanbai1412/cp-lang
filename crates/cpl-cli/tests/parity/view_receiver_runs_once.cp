// A call that heads a view chain runs once, however the chain's last
// method is dispatched: built in, from the library, or the program's own.
var calls = 0

fn make(n: Int) -> List<Int>:
    calls += 1
    [i for i in 0..n]

fn total(self: List<Int>) -> Int:
    var t = 0
    for x in self: t += x
    t

print(make(3).rev.to_list())
print(make(3).rev.collect())
print(make(3).rev.sum())
print(make(4).rev.total())
print(make(4)[1..].total())
print(calls)
