// expect: E0703 at 2
let a = [1, 2, 3]
let st = SegTree((+, max), a.map(x => (x, x)))
