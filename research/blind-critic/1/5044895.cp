// Corpus submission 5044895: cyclic permutation min-max shift (binary search + difference array)
let n: 1..=2e5 = read()
domain V = 0..n
let raw: [V]1..=n = read()
let a: [V]Int = [V] i => raw[i] - 1

fn solve_dir(xs: [V]Int) -> Int:
    fn check(x: Int) -> Bool:
        var diff: [V.cuts]Int = 0
        fn add_seg(mut diff: [V.cuts]Int, lo: Int, hi: Int):
            let l = max(0, lo)
            let r = min(n - 1, hi)
            if l <= r:
                diff[V.nth(l).l] += 1
                diff[V.nth(r).r] -= 1
        for i in V:
            var xl = i.idx - xs[i] - x
            var xr = i.idx - xs[i] + x
            while xl < 0:
                xl += n
                xr += n
            add_seg(mut diff, xl, xr)
            add_seg(mut diff, xl - n, min(xl - 1, xr - n))
        var cur = 0
        var best = 0
        for i in V:
            cur += diff[i.l]
            best max= cur
        best == n

    (0..=n).first_where(x => check(x))!

let rev_a = a.copy()
rev_a.rev
print(min(solve_dir(a), solve_dir(rev_a)))
