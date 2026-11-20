// Perf Repro #1 (P0 — causes O(N^2) timeouts on cco16p1, noi15p1, btoi17p3, cco22p3;
//                also slows ioi14p6io 10.09x, noi04p3 6.25x, dmopc21c4p2 4.16x):
// Field reads on a struct holding a `List` / array (e.g. `self.sz[x]`) and ternary
// selection between two containers (`if cond: a else: b`) emit full by-value
// `std::vector<ll> t0 = b.vals;` copies on EVERY call / iteration.
// Fix: borrow/reference projection for struct fields, ternary lvalues, and bound-method `self`.

struct Bag:
    vals: List<Int>

fn get_val(b: Bag, i: Int) -> Int:
    b.vals[i]

let n: Int = read()
var xs: List<Int> = []
for i in 0..n:
    xs.push(i)
let bag = Bag(xs)
var ans = 0
for i in 0..n:
    ans += get_val(bag, i)
print(ans)
