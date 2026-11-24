// expect: ok, output "6 3 9 4 YES"
// rule: 05.6.3
// A plain read `s[k]` on a struct with a `get` is `s.get(k)` and has its type, even beside a `lends mut` `at`; only a place goes through `at`.
struct Node:
    var n: Int
struct Pool:
    _xs: List<Node>
    pub fn at(mut self, i: Int) -> lends mut Node:
        lend mut _xs[i]
    pub fn get(self, i: Int) -> Int = _xs[i].n
struct Sparse<T>:
    _xs: List<T>
    pub fn get(self, i: Int) -> ?T = _xs.get(i)
var p = Pool([Node(5), Node(2)])
p[0].n += 1
let a: Int = p[0]
let s = Sparse([3, 7, 9])
let b = s[0] ?? -1
print(a, b, s[2] ?? 0, p[1] + p[1], yes(s[5] == None))
