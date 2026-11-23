// CF 1980F1 Field Division (easy). Bob owns everything up-right of a fountain, so column c
// gives Alice n - h cells, h the highest fountain row in columns <= c. Walk the fountains by
// column, the lowest-down first in each column; a fountain matters iff it raises h.

@!multitest
let (n, m, k): (2..=1e9, 2..=1e9, 2..=2e5) = read()
let f = read<[k](r: 1..=n, c: 1..=m)>()
var gives: [k]Int = 0
var h = 0                              // rows Bob holds in the columns walked so far
var col = 1                            // first column not yet added to Alice's area
var area = 0
for i in f.keys.sorted(by: i => (f[i].c, desc(f[i].r))):
    area += (f[i].c - col) * (n - h)
    col = f[i].c
    if f[i].r > h:
        gives[i] = 1
        h = f[i].r
print(area + (m + 1 - col) * (n - h))
print(gives)
