// The v2.2 forms the Blind Critic #5 drafts were written in.
use std.seq.{reverse}
use std.algebra.{desc}

fn key(x: Int) -> Int:
    print("key", x)
    x * 2

let n: Int = read()
var xs: List<Int> = []
for i in 0..n: xs.push((i * 7 + 3) % 10)

// `for ... if`: the body runs for what passes.
var evens: List<Int> = []
for x in xs if x % 2 == 0:
    evens.push(x)
print(evens)

// `;` between statements, also in a one-line block.
var a = 0; var b = 0
for x in xs: a += x; b += 1
print(a, b)

// `(m[k] ??= init)` is the entry: the key is worked out once.
var groups = Map<Int, List<Int>>()
(groups[key(3)] ??= []).push(7)
(groups[6] ??= []).push(8)
for x in xs: (groups[x % 3] ??= [x]).push(x)
print(groups[6], groups[0], groups[1], groups[2])

// `F64` and `{x:.2f}`.
let mean: F64 = F64(a) / F64(b)
var best: ?F64 = None
best max= mean
print("{mean:.2f} {mean:.0f} {best ?? 0.0:.3f} {mean}")

// An ordered set's ends, and the library's `reverse`, `desc`, `min(by:)`.
var seen = Set<Int>()
for x in xs: seen.add(x)
print(seen.first ?? -1, seen.last ?? -1, seen.len)
reverse(mut xs)
print(xs)
xs.sort(by: desc)
print(xs)
print(xs.min(by: x => (x - 4) * (x - 4)) ?? -1, xs.max(by: x => x % 4) ?? -1)

// A bodyless `@grader fn` is the judge's: it is read and checked, and
// never called here.
@grader fn get_distance(i: Int, j: Int) -> Int
