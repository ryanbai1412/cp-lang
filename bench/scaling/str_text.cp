// n: 512
// fixed: A
let n: Int = read()
let abs = ["ab".repeat(n / 2) for _ in 0..2]
let miss = Str("ab".repeat(n / 4) + "c")
let padded = " ".repeat(n / 2) + "x" + " ".repeat(n / 2)
let upper = "AB".repeat(n / 2)
let spaced = Str([if i % 5 == 4: ' ' else: 'x' for i in 0..n])
let words = [Str("abcd") for _ in 0..n / 4]
var acc = 0
for r in 0..4096:
    let ab = abs[r % 2]
    acc += "a".repeat(n).len // timed: strings.repeat [fixed: self]
    acc += ab.search(miss) ?? 0 // timed: strings.search
    acc += ab.rsearch(miss) ?? 0 // timed: strings.rsearch
    acc += upper.to_lower().len // timed: strings.to_lower
    acc += ab.to_upper().len // timed: strings.to_upper
    acc += padded.trim().len // timed: strings.trim
    acc += padded.trim_start().len // timed: strings.trim_start
    acc += padded.trim_end().len // timed: strings.trim_end
    acc += spaced.split_whitespace().len // timed: strings.split_whitespace
    acc += spaced.split_whitespace().len // timed: strings.split_whitespace
    acc += ",".join(words).len // timed: strings.join
    acc += ab.replace("b", "cd").len // timed: strings.replace
print(acc)
