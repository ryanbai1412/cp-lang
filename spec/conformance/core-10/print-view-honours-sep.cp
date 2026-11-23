// expect: ok, output "3\n2\n1"
// rule: 10.3.5
// `sep:` applies to any printed sequence, a reversed view included (round 23, F5).
print([1, 2, 3].rev, sep: "\n")
