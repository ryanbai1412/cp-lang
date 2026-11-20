// SYNTAX TEST "source.cp" "comments, strings, interpolation, format specs, chars and numbers"

// a comment with "quotes" and let
// <-- punctuation.definition.comment.cp
// ^^^^^^^^^ comment.line.double-slash.cp
//                ^^^^^^^^ - string
//                             ^^^ - storage

/// Doc comment for `dist`.
// <--- punctuation.definition.comment.cp
//  ^^^^^^^^^^^ comment.line.documentation.cp

let x = 1 // trailing
//      ^ constant.numeric.integer.decimal.cp
//        ^^^^^^^^^^^ comment.line.double-slash.cp

let s = "Case #{t}: {ans + 1}"
//      ^ punctuation.definition.string.begin.cp
//       ^^^^^^ string.quoted.double.cp
//             ^ punctuation.section.interpolation.begin.cp
//              ^ meta.interpolation.cp
//              ^ meta.embedded.line.cp
//               ^ punctuation.section.interpolation.end.cp
//                ^^ string.quoted.double.cp
//                       ^ keyword.operator.arithmetic.cp
//                         ^ constant.numeric.integer.decimal.cp

let fmt = "{ans:.6f} {mask:08x} {x:.5} {n:05} {n:x} {n:b} {s:>10}"
//         ^ punctuation.section.interpolation.begin.cp
//             ^ punctuation.separator.format.cp
//              ^^^ constant.other.placeholder.format.cp
//                 ^ punctuation.section.interpolation.end.cp
//                        ^ punctuation.separator.format.cp
//                         ^^^ constant.other.placeholder.format.cp
//                                ^ punctuation.separator.format.cp
//                                 ^^ constant.other.placeholder.format.cp
//                                       ^ punctuation.separator.format.cp
//                                        ^^ constant.other.placeholder.format.cp
//                                              ^ punctuation.separator.format.cp
//                                               ^ constant.other.placeholder.format.cp
//                                                    ^ punctuation.separator.format.cp
//                                                     ^ constant.other.placeholder.format.cp
//                                                          ^ punctuation.separator.format.cp
//                                                           ^^^ constant.other.placeholder.format.cp

let cond_interp = "{if ok: 1 else: 0}"
//                               ^ - punctuation.separator.format.cp
//                                 ^ constant.numeric.integer.decimal.cp

let b = "{{literal}} \n \" {x}"
//       ^^ constant.character.escape.brace.cp
//         ^^^^^^^ string.quoted.double.cp
//                ^^ constant.character.escape.brace.cp
//                   ^^ constant.character.escape.cp
//                      ^^ constant.character.escape.cp
//                          ^ meta.interpolation.cp

let n = "{names["x"].len}"
//        ^^^^^ meta.interpolation.cp
//              ^^^ string.quoted.double.cp meta.interpolation.cp string.quoted.double.cp
//                   ^^^ variable.other.property.cp

let k = "{pick(a => { let z = a; z })}" + 1
//        ^^^^ entity.name.function.call.cp
// <--- storage.type.let.cp
//                                      ^ keyword.operator.arithmetic.cp
//                                      ^ - string

let r = r"C:\raw{not}" + r
//      ^ storage.type.string.raw.cp
//        ^^^^^^^^^^^ string.quoted.double.raw.cp
//          ^^ - constant.character.escape
//              ^^^^^ - meta.interpolation
//                     ^ keyword.operator.arithmetic.cp

let unterminated = "abc
//                  ^^^ string.quoted.double.cp

let after = 1
// <--- storage.type.let.cp
// <--- - string
//          ^ constant.numeric.integer.decimal.cp

let c = 'a' + '\n' + '\''
//      ^^^ string.quoted.single.char.cp
//             ^^ constant.character.escape.cp
//                   ^^^^ string.quoted.single.char.cp

if g[r, c] == '#': continue 'rows
//            ^^^ string.quoted.single.char.cp
//                          ^^^^^ entity.name.label.cp

let big = 1e9 + 7
//        ^^^ constant.numeric.integer.decimal.cp
//              ^ constant.numeric.integer.decimal.cp

let lim = 2e5 * 1_000_000
//        ^^^ constant.numeric.integer.decimal.cp
//              ^^^^^^^^^ constant.numeric.integer.decimal.cp

let eps = 1.5e-9 + 0.5 + 1e-9
//        ^^^^^^ constant.numeric.float.cp
//                 ^^^ constant.numeric.float.cp
//                       ^^^^ constant.numeric.float.cp

let h = 0xff + 0XAB_CD + 0b1010
//      ^^^^ constant.numeric.integer.hexadecimal.cp
//             ^^^^^^^ constant.numeric.integer.hexadecimal.cp
//                       ^^^^^^ constant.numeric.integer.binary.cp

// `0..n` is a range, and `x.0` and `1.max(2)` are not fractions.
for i in 0..n: t.0 += 1.max(2)
//       ^ constant.numeric.integer.decimal.cp
//        ^^ keyword.operator.range.cp
//                    ^ constant.numeric.integer.decimal.cp
//                      ^^^ entity.name.function.method.cp
//                          ^ constant.numeric.integer.decimal.cp

let neg = -3
//        ^ keyword.operator.arithmetic.cp
//         ^ constant.numeric.integer.decimal.cp
