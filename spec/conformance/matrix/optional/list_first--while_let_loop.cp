// expect: ok, output "4"
let xs = [4, 2]
for i in 0..1:
    var left = 1
    while let y = (if left > 0: xs.first else: None):
        left -= 1
        print(y)
