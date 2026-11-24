// expect: ok, output "YES"
var ok = true
for p in 2..200:
    if not is_prime(p): continue
    for a in -3..2 * p:
        let x = ((a % p) + p) % p
        var want: ?Int = None
        for r in 0..p:
            if want == None and r * r % p == x: want = Some(r)
        ok = ok and mod_sqrt(a, p) == want
print(yes(ok))
