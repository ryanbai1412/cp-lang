// expect: ok, output "4"
let xs = [4, 2]
match xs.first:
    Some(y) => print(y)
    None => print(0)
