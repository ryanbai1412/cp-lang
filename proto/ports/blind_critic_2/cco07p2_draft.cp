// CCO 2007 P2 Snowflakes: n snowflakes, each six arm lengths around a ring. Two are twins
// if one can be turned, or flipped over and turned, onto the other. Are there twins?
// DRAFT 1: canonical form = the smallest rotation; look for a repeat in a Set.
let n: 1..=1e5 = read()
domain Flake = 0..n
domain Arm = 0..6

fn small_flakes() -> [Flake][Arm]1..=1e7:
    [Flake] _ => [Arm] _ => rand(1, 2)

let flakes: [Flake][Arm]1..=1e7 = read(gen: small_flakes)

fn canon(f: [Arm]1..=1e7) -> List<Int>:
    min([f[a.cyc(k)] for a in Arm] for k in 0..6)

fn twins() -> Bool:
    var seen = Set<List<Int>>()
    for f in flakes:
        let c = canon(f)
        if c in seen: return true
        seen.insert(c)
    false
print(if twins(): "Twin snowflakes found." else: "No two snowflakes are alike.")
