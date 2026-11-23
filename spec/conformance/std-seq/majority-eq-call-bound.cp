// expect: ok, output "1 5 10\nNone 8"
// rule: std.seq.majority
// `majority(eq:)` asks `eq` at most 3n/2 times, so it fits an equality-oracle query limit.
var calls = 0
fn eq(a: Int, b: Int) -> Bool:
    calls += 1
    a == b
let m = [1, 2, 1, 2, 1, 2, 1, 1].majority(eq: eq)!
print(m.val, m.count, calls)
calls = 0
let none = [1, 2, 3, 1, 2, 3, 1, 2].majority(eq: eq)
print(if none == None: "None" else: "Some", calls)
