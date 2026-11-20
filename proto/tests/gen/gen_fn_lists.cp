// The value of a `gen:` fn is written in the format `read()` reads: each list after its length.
let n: 2..=6 = read()
domain P = 1..=n
fn random_pals() -> [P]List<P>:
    var pals: [P]List<P> = [[] for p in P]
    for a in P:
        for b in P:
            if a < b and rand(0, 1) == 1:
                pals[a].push(b)
                pals[b].push(a)
    pals
let lists: [P]List<P> = read(gen: random_pals)
print(sum(lists[p].len for p in P))
