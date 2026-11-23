// expect: ok, output "28\n1 2 3 4 6 12\n1\n2\n3"
// rule: 06.3.4
// One `yields` fn feeds a loop, a List and a loop that breaks early; each use runs it afresh.
fn divisors(n: Int) -> yields Int:
    for d in 1..=n:
        if n % d == 0: yield d
var s = 0
for d in divisors(12): s += d
print(s)
print(List(divisors(12)))
for d in divisors(12):
    if d > 3: break
    print(d)
