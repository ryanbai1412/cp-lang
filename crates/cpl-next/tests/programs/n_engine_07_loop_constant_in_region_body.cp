// expect: ok, output "3 1\n5 1\n4"
// A loop in a region body reads constants its fn's entry sets, so the body block, run in that
// fn's frame, sees them, and a loop of its own that reads the same constant does too.
let xs = [3, 1, 5]
for x in xs:
    for j in 0..2:
        if x > 2 and j == 1:
            print(x, j)
var n = 0
for i in 0..4:
    n += 1
print(n)
