// expect: ok
// rule: 05.3.5 07.12.5
// tool: build -o -
// lacks: "-> std::vector<ll> {"
// `cand[q].partition_point(ok)` reads the row in place; the C++ must not copy it per call (acceptance run 6, row 2: cf2285E 2x TL).
let cand: List<List<Int>> = [[1, 3, 5], [2, 4]]
var t = 0
for q in 0..2:
    t += cand[q].partition_point(x => x >= 4)
print(t)
