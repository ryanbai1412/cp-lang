// n: 2048
// Shift-And steps ceil(|pat| / 64) words per element of the text, so with the pattern
// half the text it grows up to 16x at 4n where KMP's find_all, on the same input, grows 4x.
let n: Int = read()
let texts = ["ab".repeat(n / 2) for _ in 0..2]
let pats = ["ab".repeat(n / 4) for _ in 0..2]
var acc = 0
for r in 0..64:
    let (s, p) = (texts[r % 2], pats[r % 2])
    for k in s.shift_and(p): acc += k // timed: strings.shift_and
for r in 0..192:
    let (s, p) = (texts[r % 2], pats[r % 2])
    for k in s.find_all(p): acc += k // timed: strings.find_all
print(acc)
