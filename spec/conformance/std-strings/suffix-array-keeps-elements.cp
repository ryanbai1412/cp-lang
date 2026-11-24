// expect: ok, output "YES YES YES\nYES YES YES\n2 3 2"
// `SuffixArray<T>` keeps its elements and compares patterns with them by `Ord`, so it works over
// any ordered element, words and tuples included, not just ones with a `hash_token`.
fn brute<T: Ord>(s: List<T>, p: List<T>) -> List<Int>:
    [i for i in 0..s.len if i + p.len <= s.len and all(s[i + k] == p[k] for k in 0..p.len)]

fn check<T: Ord>(s: List<T>, p: List<T>) -> Bool:
    let sa = SuffixArray(s)
    sa.count(p) == brute(s, p).len and sa.find_all(p).collect().sorted() == brute(s, p)

let words = ["to", "be", "or", "not", "to", "be", "to"]
print(yes(check(words, ["to", "be"])), yes(check(words, ["to"])), yes(check(words, ["be", "to", "x"])))
let ps = [(1, -5), (0, 9), (1, -5), (0, 9), (1, -5), (2, 2)]
print(yes(check(ps, [(1, -5), (0, 9)])), yes(check(ps, [(2, 2)])), yes(check(ps, [(0, 8)])))
let big = SuffixArray([5, -3, 1_000_000_000_000, 5, -3])
print(big.rank_range([5, -3]), big.count([-3]))
