// expect: ok, output "0 0 10"
// The prelude brings in std.math's body: no compiler special case.
print(choose2(-3), choose2(1), choose2(5))
