// expect: ok, output "4\n5\n6"
// rule: 05.3.2
// A fn whose `var` copies a global is called once in the source but runs every loop turn, so codegen may not move the global into it (adversarial round 47).
// copies: 1
let base = [i + 1 for i in 0..3]
fn bump(k: Int) -> Int:
    var room = base
    room[0] += k
    room[0] + room.len
for k in 0..3:
    print(bump(k))
