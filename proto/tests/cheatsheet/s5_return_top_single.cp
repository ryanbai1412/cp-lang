// A top-level `return` ends a single-test program: nothing below it runs, and the output so far is kept.
let n: 0..=100 = read()
if n == 0:
    print("empty")
    return
let a: [0..n]1..=100 = read()
var total = 0
for x in a:
    total += x
    if total > 100:
        print("over at", x)
        return                  // out of the loop and out of the program
print(total)
