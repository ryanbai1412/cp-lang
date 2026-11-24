// expect: ok, output "3 4 4 5\n5 6 6 7\n2 3 3"
// One fold per window, whatever the op's laws: Selective `max`, Group `+`, and plain `gcd`.
let a = [3, 1, 4, 1, 5]
print(a.window_fold(2, max).collect())
print(a.window_fold(2, +).map(s => s + 1).collect())
print([4, 6, 3, 9, 12].window_fold(2, gcd).take(3).collect())
