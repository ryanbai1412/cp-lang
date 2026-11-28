// expect: ok, output "3 20 0\n2 1\n5 7\n2 1"
// `[k: v]` builds the `@role(map)` struct, or the struct the slot names, empty and then each entry
// written through its `IndexSet`; `[:]` is an empty one.
struct Pairs:
    var n: Int = 0
    var last: Int = 0
impl IndexSet<Int, Int> for Pairs:
    fn set(mut self, k: Int, v: Int):
        n += 1
        last = k * v
let m = [1: 10, 2: 20, 3: 30]
let e: Map<Int, Int> = [:]
print(m.len, m[2], e.len)
let words = ["b": 2, "a": 1]
print(words["b"], words["a"])
var mm: Map<Int, Int> = [5: 1]
mm[5] += 4
let p: Pairs = [1: 2, 7: 1]
print(mm[5], p.last)
let nested = [1: [1, 2], 2: [3]]
print(nested[1].len, nested[2].len)
