// 1913C Game with Multiset
// in: m; m lines "1 x" (add 2^x) or "2 w" (ask). out: YES/NO per ask
enum Query:
    @tag(1) Add(x: Int)
    @tag(2) Get(w: Int)

let m: Int = read()
var cnt: [30]Int = 0
repeat m:
    let q: Query = read()
    match q:
        Add(x) => cnt[x] += 1
        Get(w) =>
            var left = w
            for b in (0..30).rev:
                left -= min(cnt[b], left >> b) << b
            print(yes(left == 0))
