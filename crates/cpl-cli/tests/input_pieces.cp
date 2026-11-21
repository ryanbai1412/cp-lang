// Reads more input than the judge build takes in at once.
let n: Int = read()
var total = 0
var letters = 0
var halves = 0.0
for _ in 0..n:
    let (x, word, f): (Int, Str, F64) = read()
    total += x
    letters += word.len
    halves += f
print(total, letters, halves)
