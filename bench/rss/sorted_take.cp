// ioi20p3's shape: a list sorted by a key, its best half taken, then a 4n-cell table:
// the sorted list is freed before the table is built.
let n: 2..=1_000_000 = read()
let ups = [(gain: i * 7919 % 1_000_003, c: i) for i in 0..n]
let taken = ups.sorted(by: u => desc(u.gain)).take(n / 2).collect()
var cells: List<Int> = List.filled(4 * n, 0)
for i in 0..4 * n: cells[i] = taken[i % (n / 2)].gain + i
print(cells[4 * n - 1] + cells[0])
