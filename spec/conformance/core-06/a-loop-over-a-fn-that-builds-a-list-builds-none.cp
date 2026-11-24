// expect: ok, output "-9 7\n0 0 1 2 3\n56"
// rule: 06.3.5 13.3.3
// A `for` that always runs to its end over a fn that only builds and returns a
// list runs at each push, with no list per call; one that stops early or prints
// between walks the list, so what it prints comes after the fn has run, and so
// does a loop fused onto its pushes that stops early, as `find` does.
fn step(x: Int) -> List<Int>:
    var out: List<Int> = []
    out.push(x)
    if x % 3 == 0:
        out.push(-x)
    out
var s = 0
var odd = 0
for i in 0..7:
    for v in step(i):
        if v < 0:
            s += v
            continue
        if v % 2 == 1: odd += 1
print(s, odd + 4)
for i in 0..4:
    for v in step(i):
        if v == 3:
            print(v)
            break
        print(v, end: " ")
print(step(28).map(f => f * 2).find(q => q > 19) ?? -1)
