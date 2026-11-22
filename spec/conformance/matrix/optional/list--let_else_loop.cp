// expect: ok, output "1 2"
let xs: ?List<Int> = [1, 2]
for i in 0..1:
    let y = xs else: continue
    print(y)
