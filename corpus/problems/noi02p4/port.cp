// Source: NOI 2002 P4 (Island of Cavemen), https://dmoj.ca/problem/noi02p4
// Problem: n <= 15 cavemen on a ring of m caves; caveman i starts in cave c_i <= 100, moves p_i
//   <= 100 caves a year and lives l_i <= 1e6 years. Print the smallest m for which no two ever
//   share a cave while both live; it is at most 1e6.
// Solution: try m upward from the largest c_i. Two cavemen meet in year x when
//   (p_i - p_j) x = c_j - c_i (mod m); extended gcd gives its least solution, and m fails if
//   that is at most both lifetimes. O(answer * n^2 log m).
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
    var m = [x.cave for x in men].max()!
    while not peaceful(m): m += 1
    m

print(smallest())
