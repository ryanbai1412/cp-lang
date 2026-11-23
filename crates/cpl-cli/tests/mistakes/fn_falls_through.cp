// here: 4
// no fix: which value the missing case gives is the contestant's call (acceptance run: R0001 at run time).
fn sign(x: Int) -> Int:
    if x > 0:
        return 1
    elif x < 0:
        return -1
print(sign(3))
