// expect: ok, output "YES YES"
let empty: List<(Int, Int)> = []
print(yes(crt_all(empty) == Some((0, 1))), yes(crt_all([(13, 9)]) == Some((4, 9))))
