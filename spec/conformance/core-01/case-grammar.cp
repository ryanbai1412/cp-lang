// expect: ok, output "-1 1 1"
// rule: 01.3.1
enum Dir = L | R
fn sign(d: Dir) -> Int = match d: L => -1, R => 1
let t = true
print(sign(Dir.L), sign(Dir.R), if t: 1 else: 0)
