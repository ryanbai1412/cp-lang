// expect: ok, output "0 3 6 9\n25\n4\n10 7 4 1"
// A range's `step(k)` is its own pub member, a range again; its stride field doesn't shadow it.
print((0..10).step(3))
let r = 1..=10
print(r.step(2).sum())
print(r.step(3).len)
print(r.rev.step(3))
