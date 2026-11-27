// expect: ok, output "1 - 3\n2 x,a,1.500\n[-1] 7 -1\n4 0 | 0 4\n1 2"
// rule: R10.3.1 R10.3.7
// engines: next
// `none: d` prints each absent element of a list or array of `?T` as `d`, `Show`n with the
// line's precision; the other values print as without it.
let xs: List<?Int> = [Some(1), None, Some(3)]
print(xs, none: "-")
let a: [2]?Int = [Some(2), None]
print(a, "a", 1.5, sep: ",", none: "x", precision: 3)
let zs: List<?Int> = [None]
print("[{-1}]", 7, zs, none: -1)
let ys: List<?Int> = [Some(4), None]
write(ys, none: 0, sep: " ")
write(" | ")
let ws: [2]?Int = [None, Some(4)]
print(ws, none: 0)
print([1, 2], none: 0)
