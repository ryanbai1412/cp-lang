// v2.2 optionals: `+`, `-` and unary `-` pass `none` on, `min`/`max` and
// `min=`/`max=` skip it, and `??` can leave instead of giving a default.
let a: ?Int = 3
let b: ?Int = none
let half: ?F64 = 0.5
print((a + 1) ?? -1, (b + 1) ?? -1, (a - b) ?? -1, (1 - a) ?? -1, (-a) ?? 0, (-b) ?? 0)
print((half + 1) ?? -1.0, (half - a) ?? -1.0)
var best = 10
best min= b
best min= a
var top: ?Int = none
top max= 4
top max= b
top max= a + 4
print(best, top ?? -1, min(5, a), min(a, b) ?? -1, max(b, 7), max(1, b, a))
print(a == 3, b == 3, a != 3, b == none)

fn first_gap(xs: List<?Int>) -> Int:
    var seen = 0
    for x in xs:
        let v = x ?? return seen
        seen += v
    -1

print(first_gap([1, 2, none, 4]), first_gap([1, 2]))
var stack: List<Int> = [1, 2, 3]
var total = 0
while true:
    let t = stack.pop() ?? break
    total += t
let ys: List<?Int> = [1, none, 3, none]
for y in ys:
    let v = y ?? continue
    total += v * 10
print(total)

// `??=` fills a place once, and runs its right side only when it has to.
var calls = 0
fn fresh() -> Int:
    calls += 1
    calls * 100
var slot: ?Int = none
slot ??= fresh()
slot ??= fresh()
print(slot ?? -1, calls)
