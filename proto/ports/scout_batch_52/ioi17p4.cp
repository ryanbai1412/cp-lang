// IOI '17 P4 (The Big Prize): locate the unique diamond box among n boxes where ask(i)
// returns (more_valuable_left, more_valuable_right).
// v2.2 @memo caches the extern @grader fn directly (guaranteed 1 oracle call per index),
// eliminating the manual cache struct and memset(-1) boilerplate.
// original: corpus/submissions/5659145.cpp

@grader fn ask(i: Int) -> List<Int>

@grader
fn find_best(n: Int) -> Int:
    domain Box = 0..n

    @memo
    fn query(i: Box) -> (Int, Int):
        let res = ask(i.to_idx)
        (res[0], res[1])

    var opt_ge = -1
    while true:
        var bad = false
        var found_opt = -1
        var ans: ?Box = none

        fn search(l: Int, r: Int, cl: Int, cr: Int):
            if ans != none or bad or l > r or cl + cr == opt_ge: return
            let mid0 = (l + r) / 2
            var small = 0
            for mid in mid0..=r:
                let (ql, qr) = query(Box(mid))
                if ql == 0 and qr == 0:
                    ans = Box(mid)
                    return
                if ql + qr < opt_ge:
                    small += 1
                    continue
                if ql + qr > opt_ge:
                    bad = true
                    found_opt max= ql + qr
                    return
                search(mid + 1, r, ql, cr)
                search(l, mid0 - 1, cl, qr + small)
                return
            search(l, mid0 - 1, cl, cr + small)

        search(0, n - 1, 0, 0)
        if let hit = ans: return hit.to_idx
        assert bad
        opt_ge = found_opt
