// An `if let` over a call that returns `?T` counts its line on every test,
// matched or not, in a profile of either engine.
let exits = [-1, 2, -1, 0]

fn exit_of(s: Int) -> ?Int = if exits[s] == -1: None else: Some(exits[s])

var acc = 0
for i in 0..10:
    if let x = exit_of(i % 4): acc += x
    acc += 1
print(acc)
