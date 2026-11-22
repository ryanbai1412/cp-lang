// expect: ok, output "1 2"
// rule: 08.4.3
fn wt<R: (w: Int, ..)>(r: R) -> Int = r.w
print(wt((w: 1)), wt((w: 2, z: "a")))
