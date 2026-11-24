// Parses a string after each read of more input than the judge build takes in at once.
let n: Int = read()
var total = 0
for _ in 0..n:
    let w: Str = read()
    let (x, y) = w.split_once('-')!
    total += x.parse<Int>()! + y.parse<Int>()!
print(total)
