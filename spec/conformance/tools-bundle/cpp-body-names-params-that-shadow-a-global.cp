// expect: ok, output "12 3"
// rule: bundle.5
fn total(xs: List<Int>) -> Int = @cpp("long long s = 0; for (auto x : xs) s += x; return s;")
let xs = [3, 4, 5]
print(total(xs), xs.len)
