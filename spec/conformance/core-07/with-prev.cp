// expect: ok, output "0 5\n5 6"
// rule: 07.7.7
for (p, x) in [5, 6].with_prev: print(p ?? 0, x)
