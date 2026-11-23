// expect: ok
// rule: 04.2.27
// tool: build -o -
// lacks: "std::vector"
// `for s in Subset<V>` walks `nth(0)`, `nth(1)`, … in place, with no list of positions built first (47e918780).
domain V = 0..4
var t = 0
for s in Subset<V>:
    t += s.ord
print(t)
