// expect: ok, output "3"
domain V = 0..5
for i in 0..1:
    if let y = V.at(3):
        print(y)
