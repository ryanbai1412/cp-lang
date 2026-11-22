// expect: ok, output "4"
let xs = [4, 2]
for i in 0..1:
    let y = xs.first else: continue
    print(y)
