// expect: ok, output "5 8"
let act = Action<Int, (Int, Int)>(id: (1, 0), compose: (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1), apply: (f, val, span) => f.0 * val + f.1 * span, over: +)
let add3 = (1, 3)
let set5 = (0, 5)
print(act.apply(act.compose(set5, add3), 10, 1), act.apply(act.compose(add3, set5), 10, 1))
