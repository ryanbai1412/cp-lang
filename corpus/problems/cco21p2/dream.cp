# CCO 2021 P2 Weird Numeral System: base k with an allowed digit set D. Write each query x as
# digits from D (most significant first), or say IMPOSSIBLE.
# The last digit d must satisfy d ≡ x (mod k); then the rest represents (x - d) / k. Search that
# implicit graph from x until 0, remembering the digit on each arc.
let (k, q, n, m) = read (Int, Int, Int, Int)
let digits = read [n] Int
let by_rem = digits.group(by: d => d mod k)
for x in read [q] Int:
    let paths = bfs(y => by_rem[y mod k].map(d => (to: (y - d) / k, digit: d)), from: [x], until: 0, hashed: true)
    match paths.arcs_to(0):
        Some(arcs) => print(arcs.rev.map(.digit))
        None => print("IMPOSSIBLE")
