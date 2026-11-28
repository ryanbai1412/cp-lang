// expect: ok, output "3 2"
fn g(n: Int = 3) -> Int: n
fn g(; from: Str) -> Int: from.len
print(g(), g(from: "ab"))
