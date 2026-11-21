// SYNTAX TEST "source.cp" "Syntax Batches #7-#9: wide ints, Dir8, if const, unchecked, guarded let, Set/Map members"

@!multitest
// <-- meta.directive.cp punctuation.definition.directive.cp
//^^^^^^^^^ meta.directive.cp keyword.control.directive.cp

var big: I128 = 0
//       ^^^^ support.type.builtin.cp
let u: U64 = 1
//     ^^^ support.type.builtin.cp

for d in Dir8.all():
//       ^^^^ support.type.stdlib.cp
    print(d.cw)
//          ^^ variable.other.property.cp

if const FAST:
// <-- keyword.control.if.cp
// ^^^^^ storage.modifier.const.cp
    print(1)

unchecked:
// <--------- keyword.other.unsafe.unchecked.cp
    total += a.raw[i]
//             ^^^ keyword.other.unsafe.raw.cp

while let x = xs.pop(), x > 1:
// <----- keyword.control.while.cp
//    ^^^ storage.type.let.cp
//                    ^ punctuation.separator.comma.cp
    print(x)
if let z = xs.pop(), z > 0: print(z) else: print("empty")
// <-- keyword.control.if.cp
// ^^^ storage.type.let.cp

print(s.first, s.first(), m.pop_first(), s.find_all("bc"), step((r, c), d))
//      ^^^^^ variable.other.property.cp
//               ^^^^^ entity.name.function.method.cp
//                          ^^^^^^^^^ entity.name.function.method.cp
//                                         ^^^^^^^^ entity.name.function.method.cp
//                                                         ^^^^ support.function.builtin.cp
