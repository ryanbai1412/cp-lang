// NOI 2002 Island of Cavemen: caveman i starts in cave c[i] of a ring of m
// caves, walks p[i] caves a year and lives l[i] years. Find the smallest m
// (at least the largest starting cave) for which no two living cavemen ever
// share a cave. The answer is at most 1e6.
// Two cavemen meet in year x when (p[i] - p[j]) x = c[j] - c[i] (mod m): one
// linear congruence per pair, and only its smallest solution matters.
// original: corpus/submissions/3741279.cpp
let n: 1..=15 = read()
struct Caveman:
    cave: 1..=100
    pace: 1..=100
    life: 1..=1e6
let men: [n]Caveman = read()

// g = gcd(a, b) and x with a x + b y = g. (std.math.ext_gcd, inlined: the
// prototype compiler cannot import it yet.)
fn ext_gcd(a: Int, b: Int) -> (g: Int, x: Int):
    var r0 = a
    var r1 = b
    var x0 = 1
    var x1 = 0
    while r1 != 0:
        let q = r0 / r1
        let r2 = r0 - q * r1
        let x2 = x0 - q * x1
        r0 = r1
        r1 = r2
        x0 = x1
        x1 = x2
    (g: r0, x: x0)

// Smallest x >= 0 with a x = b (mod m), if any.
fn first_solution(a: Int, b: Int, m: Int) -> ?Int:
    let e = ext_gcd(a % m, m)
    if b % e.g != 0: return None
    let step = m / e.g
    (e.x % step) * ((b / e.g) % step) % step

fn peaceful(m: Int) -> Bool:
    for i in 0..n:
        for j in 0..i:
            let (a, b) = (men[i], men[j])
            if let year = first_solution(a.pace - b.pace, b.cave - a.cave, m):
                if year <= min(a.life, b.life): return false
    true

fn smallest() -> Int:
    var m = max(x.cave for x in men)
    while not peaceful(m): m += 1
    m

print(smallest())
