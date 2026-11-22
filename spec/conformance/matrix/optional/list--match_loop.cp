// expect: ok, output "1 2"
let xs: ?List<Int> = [1, 2]
for i in 0..1:
    match xs:
        Some(y) => print(y)
        None => print(0)
