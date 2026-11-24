// expect: E0302
// rule: 04.4.24
// has: "check each element into the range: `[x for x in zs]`"
// `zs` holds `500`; storing it as `ys` would put it in a `0..100` cell unchecked.
var ys: List<0..100> = [1]
var zs: List<Int> = ys
zs.push(500)
ys = zs // <- here
print(ys[1])
