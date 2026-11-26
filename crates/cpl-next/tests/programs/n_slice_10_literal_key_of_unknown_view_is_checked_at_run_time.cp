// expect: R0101
// rule: R07.4.5
// input: "1"
// A view whose keys the checker does not know takes a literal key and checks it as it runs.
let xs = [5, 6, 7, 8]
let lo = read<Int>()
let w = xs[lo..4]
print(w[2], w[3])
print(w[0]) // <- here
