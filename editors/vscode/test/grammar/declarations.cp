// SYNTAX TEST "source.cp" "CP Lang 0.2.2 declarations, decorators and imports"

use std.graph
// <--- keyword.control.import.cp
//  ^^^ entity.name.namespace.cp
//     ^ punctuation.accessor.dot.cp
//      ^^^^^ entity.name.namespace.cp

use lib.treap.{Treap}
//      ^^^^^ entity.name.namespace.cp
//             ^^^^^ entity.name.type.cp

use lib.graph.flow as fl // alias
//            ^^^^ entity.name.namespace.cp
//                 ^^ keyword.control.import.as.cp
//                    ^^ entity.name.namespace.cp
//                       ^^^^^^^^ comment.line.double-slash.cp

use std.segtree.*
//              ^ keyword.operator.glob.cp

@multitest
// <- punctuation.definition.decorator.cp
// <~--------- entity.name.function.decorator.cp

@memo
// <- punctuation.definition.decorator.cp
// <~---- meta.decorator.cp entity.name.function.decorator.cp

@brute(main)
// <~----- entity.name.function.decorator.cp
//     ^^^^ - entity.name.function

@grader fn restore(n: Int) -> Int:
// <~------ entity.name.function.decorator.cp
//      ^^ storage.type.function.cp
//         ^^^^^^^ entity.name.function.cp
//                 ^ variable.parameter.label.cp

@checker fn check():
// <~------- entity.name.function.decorator.cp
//       ^^ storage.type.function.cp
//          ^^^^^ entity.name.function.cp

@interactor fn judge(sol: Interactor):
// <~---------- entity.name.function.decorator.cp
//          ^^ storage.type.function.cp
//             ^^^^^ entity.name.function.cp
//                   ^^^ variable.parameter.label.cp
//                        ^^^^^^^^^^ support.type.builtin.cp

@gen fn make_case(size: Int):
// <~--- entity.name.function.decorator.cp
//   ^^ storage.type.function.cp
//      ^^^^^^^^^ entity.name.function.cp

@tags('.', '*', '1'..='9')
// <~---- entity.name.function.decorator.cp
//    ^^^ string.quoted.single.char.cp
//                 ^^^ keyword.operator.range.cp

fn solve_dist(src: V) -> [V]?Int:
// <-- storage.type.function.cp
// ^^^^^^^^^^ entity.name.function.cp
//            ^^^ variable.parameter.label.cp
//                 ^ entity.name.type.cp
//                    ^^ keyword.operator.arrow.cp
//                          ^ keyword.operator.optional.cp
//                           ^^^ support.type.builtin.cp

fn sq(x: Int) -> Int = x * x
// ^^ entity.name.function.cp
//    ^ variable.parameter.label.cp
//                   ^ keyword.operator.assignment.cp
//                       ^ keyword.operator.arithmetic.cp

pub fn link(mut g: Dsu<V>, r: Road; strict: Bool = false):
// <--- storage.modifier.pub.cp
//     ^^^^ entity.name.function.cp
//          ^^^ storage.modifier.mut.cp
//              ^ variable.parameter.label.cp
//                 ^^^ support.type.stdlib.cp
//                    ^ punctuation.definition.generic.begin.cp
//                      ^ punctuation.definition.generic.end.cp
//                         ^ variable.parameter.label.cp
//                                  ^^^^^^ variable.parameter.label.cp
//                                          ^^^^ support.type.builtin.cp

@test fn check_tree():
// <~---- entity.name.function.decorator.cp
//    ^^ storage.type.function.cp
//       ^^^^^^^^^^ entity.name.function.cp

struct Road(u: V, v: V, w: Int = 0):
// <------ storage.type.struct.cp
//     ^^^^ entity.name.type.struct.cp
//          ^ variable.parameter.label.cp
//                      ^ variable.parameter.label.cp

    fn update(mut self, i: I, x: Int):
//     ^^^^^^ entity.name.function.cp
//            ^^^ storage.modifier.mut.cp
//                ^^^^ variable.language.self.cp
//                      ^ variable.parameter.label.cp

    @get fn len(self) -> Int = self.n
//   ^^^ entity.name.function.decorator.cp
//          ^^^ entity.name.function.cp
//                             ^^^^ variable.language.self.cp
//                                  ^ variable.other.property.cp

enum Dir = L | R | U | D
// <---- storage.type.enum.cp
//   ^^^ entity.name.type.enum.cp
//         ^ entity.name.type.cp
//           ^ keyword.operator.bitwise.cp

type Kids = List<Bar>
// <---- storage.type.alias.cp
//   ^^^^ entity.name.type.alias.cp
//          ^^^^ support.type.builtin.cp
//               ^^^ entity.name.type.cp

const MOD = 998_244_353
// <----- storage.modifier.const.cp
//    ^^^ variable.other.constant.cp
//          ^^^^^^^^^^^ constant.numeric.integer.decimal.cp

let y = x % MOD
//          ^^^ constant.other.caps.cp

trait Monoid<T>:
// <----- storage.type.trait.cp
//    ^^^^^^ entity.name.type.trait.cp

    @law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
//  ^ punctuation.definition.decorator.cp
//   ^^^ entity.name.function.decorator.cp
//       ^^^^^ entity.name.function.call.cp
//             ^ variable.parameter.label.cp
//                                 ^^ entity.name.function.call.cp

impl Ord for Road:
// <---- storage.type.impl.cp
//   ^^^ entity.name.type.cp
//       ^^^ keyword.control.for.cp
//           ^^^^ entity.name.type.cp

fn bsearch(lo: Int, hi: Int) -> Int:
// ^^^^^^^ entity.name.function.cp

    while l < r:
//  ^^^^^ keyword.control.while.cp

        invariant l <= r
//      ^^^^^^^^^ keyword.other.contract.invariant.cp

        invariant = l
//      ^^^^^^^^^ - keyword

    dsu.undo_to(snap)
//      ^^^^^^^ entity.name.function.method.cp
