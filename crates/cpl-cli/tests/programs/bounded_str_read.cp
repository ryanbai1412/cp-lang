// `Str(len: …)` checks the length of a read word, `of` its alphabet; the
// bounds may be runtime values and the two compose (R02.3.2).
let n = read<Int>()
let exact = read<Str(len: n..=n)>()
let bits: Str(len: 1..=n) of "01" = read()
let moves = read<Str of "UDLR">()
let words: [2]Str(len: 1..=5) = read()
print(exact, exact.len, bits, bits.len, moves)
print(words[0] + words[1])
