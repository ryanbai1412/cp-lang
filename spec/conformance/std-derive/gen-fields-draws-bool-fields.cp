// expect: ok, output "YES"
// `gen<Bool>()` is a fair coin, so `gen_fields` draws a `Bool` field like any
// other: over many draws both values turn up.
use std.derive.gen_fields
struct F(on: Bool, k: 1..=2)
var heads = 0
var tails = 0
for _ in 0..200:
    let f = gen_fields<F>()
    if f.on:
        heads += 1
    else:
        tails += 1
    assert 1 <= f.k <= 2
print(yes(heads > 0 and tails > 0 and heads + tails == 200))
