// expect: ok, output "6"
// rule: 03.1.9
// input: "2 3\nabc\ndef\n"
// Two dimensions bounded by the input apart may be bounded jointly (`n * m <= 1e6`), so their product is no E0904 peak.
let (n, m): (1..=1e6, 1..=1e6) = read()
let cells: 1..=1e6 = n * m
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Char = read()
print(cells)
