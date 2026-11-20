// CCO '22 P2 — Rainy Markets
// original: corpus/submissions/4801882.cpp
//
// Greedily pack people at market i into bus i, then bus i+1, then umbrellas.
// Maintain a run-length-encoded stack of unused umbrella capacity from earlier
// markets (truncated to the current spillover into bus i+1). Whenever umbrellas
// fall short, pull capacity from the stack and shift intermediate markets left
// by one via a difference array.

let n: 2..=1_000_000 = read()
domain Bus = 0..n
domain Mkt = 0..n - 1
let bus: [Bus]Int = read()
let ppl: [Mkt]Int = read()
let umbr: [Mkt]Int = read()

struct Run:
    at: Mkt
    cnt: Int
    pref: Int

var stack: List<Run> = []
var total = 0

fn truncate(cap: Int):
    if total <= cap: return
    while let top = stack.last and top.pref >= cap:
        stack.pop()
    if stack.len > 0:
        stack.last!.cnt = cap - stack.last!.pref
    total = cap

var left_bus: [Mkt]Int = 0
var rain: [Mkt]Int = 0
var right_bus: [Mkt]Int = 0
var diff: [Bus]Int = 0
var carry = 0

for i in Mkt:
    var x = min(ppl[i], bus[Bus(i.idx)] - carry)
    let z = min(bus[Bus(i.idx + 1)], ppl[i] - x)
    var y = ppl[i] - x - z
    if y <= umbr[i]:
        stack.push(Run(i, umbr[i] - y, total))
        total += umbr[i] - y
    else:
        var need = y - umbr[i]
        if total < need:
            print("NO")
            return
        total -= need
        while need > 0:
            let top = stack.last!
            let take = min(need, top.cnt)
            need -= take
            rain[top.at] += take
            right_bus[top.at] -= take
            diff[Bus(top.at.idx + 1)] += take
            diff[Bus(i.idx)] -= take
            stack.last!.cnt -= take
            if stack.last!.cnt == 0: stack.pop()
        x += y - umbr[i]
        y = umbr[i]
    left_bus[i] = x
    rain[i] = y
    right_bus[i] = z
    carry = z
    truncate(z)

var shift = 0
var used = 0
for i in Mkt:
    shift += diff[Bus(i.idx)]
    left_bus[i] += shift
    right_bus[i] -= shift
    used += rain[i]

print("YES")
print(used)
for i in Mkt: print(left_bus[i], rain[i], right_bus[i])
