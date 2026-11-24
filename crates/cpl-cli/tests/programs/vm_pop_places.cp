// `pop` as a value from any list place: a local, a global, a field, a
// list or array cell, a map entry, and through `mut` and nested structs.
struct S: xs: List<Str>, n: Int
struct T: s: S, k: Int
var g = S(["a", "b"], 0)
print(g.xs.pop()!, g.xs)
var t = T(S(["c", "d"], 1), 2)
print(t.s.xs.pop()!, t.s.xs.len)
var ls: List<List<Str>> = [["e", "f"], ["g"]]
print(ls[0].pop()!, ls[1].pop()!, yes(ls[1].pop() == None), ls)
var arr: [3]List<Str> = []
arr[1].push("h")
print(arr[1].pop()!, arr[1].len)
var m = Map<Int, List<Str>>()
m[1] = ["i", "j"]
print(m[1].pop()!, m[1])
fn f(mut s: S) -> Str = s.xs.pop()!
var u = S(["k", "l"], 0)
print(f(mut u), u.xs)
fn h() -> Str:
    var lt = T(S(["m", "n"], 1), 2)
    var lls: List<List<Str>> = [["o"]]
    var lxs: List<Str> = ["p"]
    let local: [2]Str = "q"
    return lt.s.xs.pop()! + lls[0].pop()! + lxs.pop()! + local[1] + "{lxs.len}"
print(h())
var gs: List<S> = [S(["r", "s"], 0)]
for i in 0..2: print(gs[0].xs.pop()!)
var mo = Map<Int, List<Str>>()
(mo[3] ??= ["v", "w"]).pop()
print(mo[3])
