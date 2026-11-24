// expect: R0101 at 10
// input: "4"
// rule: 07.4.14
// `w.cut_after(i)` needs `i` to be a key of `w`: a key of the parent outside the slice has no cut in `w`'s walk.
domain V = 1..=4
let a = [V] v => Int(v)
let w = a[V(2)..=V(3)]
let k: Int = read()
let i = V(k)
print(yes(w.cut_after(i) == i.r))
