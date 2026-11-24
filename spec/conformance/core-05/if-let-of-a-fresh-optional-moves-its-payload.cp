// expect: ok, output "3 YES"
fn f(n: Int) -> ?List<Int> = if n > 0: Some([n for _ in 0..n]) else: None
fn g(n: Int) -> ?List<Int>:
    if let got = f(n): return Some(got)
    None
let n = match g(3):
    Some(xs) => xs.len
    None => 0
print("{n} {yes(g(0) == None)}")
