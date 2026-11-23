// An operator alone is `(a, b) => a op b` wherever a value goes: a default, a record field, a `let`.
fn ap(f: (Int, Int) -> Int = (+)) -> Int = f(3, 4)
print(ap(), ap((a, b) => a * b), ap(max))
let g: (Int, Int) -> Int = (*)
print(g(5, 6))
let ops = (count: +, n: 2)
print(ops.n, ops.count(2, 5))
// std's `Action.new` defaults `compose` and `over` to `(+)` (corpus etsr23r4p2).
domain Slot = 0..=2
let sums = Monoid(id: [Slot] _ => 0, op: (x: [Slot]Int, y: [Slot]Int) => [Slot] s => x[s] + y[s])
let raise = Action<[Slot]Int, Int>.new(
    apply: (k: Int, x: [Slot]Int, len: Int) => [Slot] s => x[Slot.at(Int(s) + k) ?? Slot.last!],
    over: sums,
)
domain I = 1..=3
var st = SegTree(sums, [I] i => [Slot] s => Int(s) + Int(i), act: raise)
print(st.fold(I.first!..=I.last!)[Slot.first!])
st.apply(I.first!..=I.last!, 1)
st.apply(I.first!..=I.first!, 1)
print(st.fold(I.first!..=I.last!)[Slot.first!])
