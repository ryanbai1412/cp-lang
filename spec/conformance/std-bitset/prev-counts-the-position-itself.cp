// expect: ok, output "YES"
domain D = 0..200
var b = Bitset<D>.new()
let marks = [0, 5, 63, 64, 130, 199]
for m in marks: b.add(D(m))
var ok = true
for i in 0..200:
    var want: ?D = None
    for m in marks:
        if m <= i: want = Some(D(m))
    ok = ok and b.prev(D(i)) == want
print(yes(ok))
