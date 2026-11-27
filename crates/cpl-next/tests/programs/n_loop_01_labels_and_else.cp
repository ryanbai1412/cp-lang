// expect: ok, output "none\n1 2\ndone 3\nfound\n4"
// A loop's `else:` runs when it ended without `break`; `break 'label` leaves every loop up to
// the labelled one, skipping the `else`s of those it leaves (R01.7.2).
for x in [1, 2]:
    if x > 5: break
else:
    print("none")
'outer: for r in 1..=3:
    for c in 1..=3:
        if r + c == 3:
            print(r, c)
            break 'outer
    else:
        print("inner else")
var n = 0
'a: for i in 0..3:
    for j in 0..3:
        for k in 0..3:
            n += 1
            if k == 2: break 'a
        else:
            print("never")
else:
    print("never")
print("done", n)
var i = 0
while i < 10:
    i += 1
    if i == 4: break
else:
    print("not found")
print("found")
print(i)
