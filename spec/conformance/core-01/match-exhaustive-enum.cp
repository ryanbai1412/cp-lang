// expect: ok, output "2"
// rule: 01.7.3
enum Dir = L | R
let d = Dir.R
match d:
    L => print(1)
    R => print(2)
