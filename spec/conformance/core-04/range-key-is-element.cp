// expect: ok, output "3 4\nYES\n2 2\n3 3\n4 4"
// rule: 04.4.31
let r = 2..5
print(r[3], r[4])
print(yes(r.keys == r))
for (k, x) in r.items:
    print(k, x)
