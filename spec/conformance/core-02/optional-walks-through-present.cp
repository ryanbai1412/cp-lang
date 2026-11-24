// expect: ok, output "4 7 8\n7 8"
// rule: 02.4.9
// A zero-or-one value walks as a one-slot list: `[x].present()` (R02.4.9).
let up: ?Int = Some(4)
let root: ?Int = None
let kids = [7, 8]
print(chain([up].present(), kids))
print(chain([root].present(), kids))
