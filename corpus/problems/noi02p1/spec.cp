// NOI 2002 "Legend of the Galactic Heroes": columns of ships; M i j puts i's
// column behind j's, C i j asks how many ships sit between i and j. A DSU with
// potentials: a ship's potential is its place in the column.
domain Ship = 1..=30000
enum Cmd = M(i: Ship, j: Ship) | C(i: Ship, j: Ship)
var line = PotentialDsu<Ship>()
let t = read<1..=500_000>()
for cmd in read<[t]Cmd>():
    match cmd:
        M(i, j) => line.unite(line.find(j).root, line.find(i).root, line.size(j))
        C(i, j) => print(line.diff(i, j).map(d => abs(d) - 1) ?? -1)
