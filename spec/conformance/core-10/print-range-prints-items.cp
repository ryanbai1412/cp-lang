// expect: ok, output "1 2 3\n0,1"
// rule: 10.3.1
// A range is a `Seq`, so `print` writes its elements joined by `sep`, not `1..4` (round 27, F7).
print(1..=3)
print(0..2, sep: ",")
