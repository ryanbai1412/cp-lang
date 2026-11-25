// expect: ok, output "0 4\n1"
// input: "3\n"
// Function values inside a type argument, mutually recursive through the list.
var hs: List<Int -> Int> = []
fn even(x: Int) -> Int = if x == 0: 1 else: hs[1](x - 1)
fn odd(x: Int) -> Int = if x == 0: 0 else: hs[0](x - 1)
hs.push(even)
hs.push(odd)
let k: Int = read()
hs.push(x => x + k)
print("{hs[0](7)} {hs[2](1)}")
var g = hs[2]
if k > 1: g = hs[0]
print(g(4))
