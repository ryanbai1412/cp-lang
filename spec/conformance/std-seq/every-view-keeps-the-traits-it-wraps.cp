// expect: ok, output "3 3 6 5 2 9 3 2 2"
// rule: 07.4.2
// A view is `Bidir` when what it wraps is: `.last` through a `Bidir` bound reads filter, map, items, a slice, step, rebase and nested views from the back.
fn lst<S: Bidir<T>, T>(s: S) -> ?T = s.last
let ys = [3, -1, 4, -5, 9, 2]
let pos = (x: Int) => x > 0
print(
    lst(ys.rev.filter(pos))!,
    lst(ys.filter(pos).rev)!,
    lst(ys.rev.map(x => x * 2))!,
    lst(ys.items)!.0,
    lst(ys.drop(1))!,
    lst(ys.step(2))!,
    lst(ys.rev.drop(1).filter(pos))!,
    lst(ys.rebase)!,
    lst(ys.drop(1).step(2))!,
)
