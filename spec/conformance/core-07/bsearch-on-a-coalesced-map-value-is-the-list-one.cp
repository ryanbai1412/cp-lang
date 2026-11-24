// expect: ok, output "2 1"
// rule: 07.12.5
// A receiver typed through `m.get(k) ?? []` or `if let` picks `[D]T.bsearch`, never `Range.bsearch` by declaration order.
var m = Map<Int, List<Int>>()
m[0] = [1, 5, 9]
var s = 0
for hi in 0..4:
    let l = m.get(0) ?? []
    s += l.bsearch(x => x >= hi)
let pos: Map<Int, List<Int>> = [1: [1, 2, 3]]
if let ps = pos.get(1):
    print(s, ps.bsearch(p => p > 1))
