// Codeforces 2157 H: Keygen 3 (dream).
// A bitonic permutation is fixed by which of 1..n-1 sit left of the peak n.
// Strip its identity prefix: the core left over starts above its first slot,
// so its rising half has no fixed point and its falling half at most one, and a
// core of length L has at most (L+1)/2 cycles. With d = n - m, every answer is
// 1..n-L followed by a core of length L = min(n, 2d+1) with L - d cycles. Up to
// L = 20 we list them all; past that there are always more than 2000, and, as in
// the accepted solutions, we sample one-cycle cores of length d+1 (unproved here).
let (n, m): (1..=100, 1..=100) = read()
let d = n - m

fn core(len: Int, left: Set<Int>) -> List<Int>:
    let below = 1..len
    below.filter(v => v in left) + [len] + below.filter(v => v not in left).rev

fn padded(c: List<Int>) -> List<Int>:
    let k = n - c.len
    (1..=k).to_list() + c.map(v => v + k)

let full = min(n, 2 * d + 1)
let cores = if full <= 20:
        (1..full).subsets().map(s => core(full, s)).filter(c => c.cycle_count() == full - d).take(2000).to_list()
    else:
        var seen = Set<List<Int>>()
        while seen.len < 2000:
            let c = core(d + 1, random_subset(1..d + 1))
            if c.cycle_count() == 1: seen.add(c)
        seen.to_list()
print(cores.len)
for c in cores: print(padded(c))
