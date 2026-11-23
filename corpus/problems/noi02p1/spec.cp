// NOI 2002 "Legend of the Galactic Heroes": columns of ships; M i j puts i's
// column behind j's, C i j asks how many ships sit between i and j. A DSU with
// potentials: a ship's potential is its place in the column, and each set
// remembers its front ship, since the DSU's root is whichever set was larger.
domain Ship = 1..=30000
// payloads are `1..=30000`, not `Ship`: a domain-typed payload misreads the tag (compiler worklog)
enum Cmd = M(i: 1..=30000, j: 1..=30000) | C(i: 1..=30000, j: 1..=30000)
var line = PotentialDsu<Ship>()
var head: [Ship]?Ship = None
fn front(v: Ship) -> Ship:
    let r = line.find(v).root
    head[r] ?? r
let t = read<1..=500_000>()
for cmd in read<[t]Cmd>():
    match cmd:
        M(i, j) =>
            let (fi, fj) = (front(Ship(i)), front(Ship(j)))
            line.unite(fj, fi, line.size(Ship(j)))
            head[line.find(fi).root] = Some(fj)
        C(i, j) => print(line.diff(Ship(i), Ship(j)).map(d => abs(d) - 1) ?? -1)
