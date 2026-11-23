// SYNTAX TEST "source.cp" "CP Lang 0.2.2 keywords and operators"

let n = read()
// <--- storage.type.let.cp
//    ^ keyword.operator.assignment.cp
//      ^^^^ support.function.builtin.cp

var best = 0
// <--- storage.type.var.cp
//         ^ constant.numeric.integer.decimal.cp

if a < b and not done or c >= d:
// <-- keyword.control.if.cp
//   ^ keyword.operator.comparison.cp
//       ^^^ keyword.operator.logical.and.cp
//           ^^^ keyword.operator.logical.not.cp
//                    ^^ keyword.operator.logical.or.cp
//                         ^^ keyword.operator.comparison.cp
//                             ^ punctuation.separator.colon.cp

elif x == y: continue
// <---- keyword.control.elif.cp
//     ^^ keyword.operator.comparison.cp
//           ^^^^^^^^ keyword.control.continue.cp

else: return None
// <---- keyword.control.else.cp
//    ^^^^^^ keyword.control.return.cp
//           ^^^^ constant.language.none.cp

for i in 0..blocks.len:
// <--- keyword.control.for.cp
//    ^^ keyword.control.in.cp
//        ^^ keyword.operator.range.cp

for v in 1..=n: total += w[v]
//       ^ constant.numeric.integer.decimal.cp
//        ^^^ keyword.operator.range.cp
//                    ^^ keyword.operator.assignment.compound.cp

while let (d, u) = pq.pop():
// <----- keyword.control.while.cp
//    ^^^ storage.type.let.cp
//                    ^^^ entity.name.function.method.cp

    break 'outer
//  ^^^^^ keyword.control.break.cp
//        ^^^^^^ entity.name.label.cp

match op:
// <----- keyword.control.match.cp

    Add(l, r) => st.add(l, r)
//  ^^^ entity.name.type.cp
//            ^^ keyword.operator.fat-arrow.cp
//                  ^^^ entity.name.function.method.cp

assert lo <= hi, "empty"
// <------ keyword.control.assert.cp
//        ^^ keyword.operator.comparison.cp
//               ^^^^^^^ string.quoted.double.cp

assert n >= 1 and sum(a) <= 2e5
// <------ keyword.control.assert.cp
//       ^^ keyword.operator.comparison.cp
//            ^^^ keyword.operator.logical.and.cp
//                ^^^ support.function.builtin.cp

let u = opt ?? return (None, None)
//          ^^ keyword.operator.coalesce.cp
//             ^^^^^^ keyword.control.return.cp
//                     ^^^^ constant.language.none.cp

yield from walk(child)
// <----- keyword.control.yield.cp
//    ^^^^ keyword.control.yield.from.cp
//         ^^^^ entity.name.function.call.cp

yield from
// <----- keyword.control.yield.cp
//    ^^^^ - keyword

best max= x
//   ^^^^ keyword.operator.assignment.compound.cp

best min= x
//   ^^^^ keyword.operator.assignment.compound.cp

let m = max(a, b) == min(a, b)
//      ^^^ support.function.builtin.cp
//                ^^ keyword.operator.comparison.cp
//                   ^^^ support.function.builtin.cp

let p = x ** 2 ^ y % m
//        ^^ keyword.operator.arithmetic.cp
//             ^ keyword.operator.bitwise.cp
//                 ^ keyword.operator.arithmetic.cp

x **= 2
//^^^ keyword.operator.assignment.compound.cp

mask <<= 1
//   ^^^ keyword.operator.assignment.compound.cp

(adj[k] ??= []).push(x)
//      ^^^ keyword.operator.assignment.compound.cp
//              ^^^^ entity.name.function.method.cp

let s = a << 3 | b >> 1 & ~c
//        ^^ keyword.operator.bitwise.shift.cp
//             ^ keyword.operator.bitwise.cp
//                 ^^ keyword.operator.bitwise.shift.cp
//                      ^ keyword.operator.bitwise.cp
//                        ^ keyword.operator.bitwise.cp

let d = dist[v] ?? -1
//              ^^ keyword.operator.coalesce.cp
//                 ^ keyword.operator.arithmetic.cp

let f = first_true(0..=hi, ok)!
//      ^^^^^^^^^^ support.function.stdlib.cp
//                            ^ keyword.operator.unwrap.cp

let g = a.find(x)?
//        ^^^^ entity.name.function.method.cp
//               ^ keyword.operator.optional.cp

if x != y: print(true, false)
//   ^^ keyword.operator.comparison.cp
//         ^^^^^ support.function.builtin.cp
//               ^^^^ constant.language.boolean.true.cp
//                     ^^^^^ constant.language.boolean.false.cp

let t = f::<Int>()
//      ^ entity.name.function.call.cp
//       ^^ punctuation.accessor.double-colon.cp
//         ^ punctuation.definition.generic.begin.cp
//          ^^^ support.type.builtin.cp
//             ^ punctuation.definition.generic.end.cp

if const D.K <= 64:
// ^^^^^ storage.modifier.const.cp
//       ^ entity.name.type.cp
//         ^ entity.name.type.cp

// Words that are contextual keywords stay plain names elsewhere.
let from = as + invariant
//  ^^^^ - keyword
//         ^^ - keyword
//              ^^^^^^^^^ - keyword

let first = xs.first
//  ^^^^^ - support.function
//             ^^^^^ variable.other.property.cp

let x = sat.var() + s.type
//         ^ punctuation.accessor.dot.cp
//          ^^^ entity.name.function.method.cp
//                    ^^^^ variable.other.property.cp
