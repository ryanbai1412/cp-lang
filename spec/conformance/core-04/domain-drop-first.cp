// expect: ok, output "2 3 4"
// rule: 04.2.20
// A domain is a `Seq`, so `E.drop(1)` walks every position after the first.
domain E = 1..=4
print([Int(e) for e in E.drop(1)])
