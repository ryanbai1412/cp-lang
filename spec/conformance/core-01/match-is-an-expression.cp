// expect: ok, output "20"
// rule: 01.10.3
// A `match` can give a value; every arm gives one.
let d = 2
let x = match d:
    1 => 10
    2 => 20
    _ => 0
print(x)
