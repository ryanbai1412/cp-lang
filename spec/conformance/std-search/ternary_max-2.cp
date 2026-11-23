// expect: ok, output "YES"
let f = [0, 1, 2, 2, 1]
print(yes(ternary_max(0..5, i => f[i]) == Some(2)))
