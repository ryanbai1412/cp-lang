// expect: ok, output "777 ok"
// `first_true` on a range of n positions calls `ok` at most ceil(log2(n + 1)) times, 17 for n = 100000, so an interactor's query budget can be counted (rebuild round 33 S10).
var calls = 0
fn ok(c: Int) -> Bool:
    calls += 1
    c >= 777
let c = (0..100000).first_true(ok)
print(c!, if calls <= 17: "ok" else: "over")
