// expect: ok, output "1 1 0 0 1 1 3 4"
// rule: 05.10.13
struct C:
    @cache memo: List<Int>
    k: Int
    v: Int

let a = C(memo: [1, 2], k: 3, v: 4)
let b = C(memo: [9], k: 3, v: 4)
let c = C(memo: [0], k: 3, v: 5)
var m: HashMap<C, Int> = HashMap()
m[a] = 42
var s: Set<C> = Set()
let _ = s.insert(a)
print(
    if a == b: 1 else: 0,
    if not (a < b) and not (b < a): 1 else: 0,
    if c < a: 1 else: 0,
    if a > c: 1 else: 0,
    if b in s: 1 else: 0,
    if m.get(b) == Some(42): 1 else: 0,
    a,
)
