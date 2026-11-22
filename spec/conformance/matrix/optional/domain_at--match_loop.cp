// expect: ok, output "3"
domain V = 0..5
for i in 0..1:
    match V.at(3):
        Some(y) => print(y)
        None => print(0)
