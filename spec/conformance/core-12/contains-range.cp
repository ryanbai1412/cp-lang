// expect: ok, output "YES YES"
// rule: 12.4.7
print(yes(3 in 1..5), yes([1, 2, 3].any(x => x == 3)))
