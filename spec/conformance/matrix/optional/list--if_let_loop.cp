// expect: ok, output "1 2"
let xs: ?List<Int> = [1, 2]
for i in 0..1:
    if let y = xs:
        print(y)
