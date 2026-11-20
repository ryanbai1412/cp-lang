// SYNTAX TEST "source.cp" "declarations, decorators and imports"

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

@multitest(sum: n <= 2e5)
// <- punctuation.definition.decorator.cp
// <~--------- entity.name.function.decorator.cp
//         ^^^ variable.parameter.label.cp
//                ^^ keyword.operator.comparison.cp
//                   ^^^ constant.numeric.integer.decimal.cp

@sample("3\n1 2 3\n", "6\n")
// <~------ entity.name.function.decorator.cp
//       ^ string.quoted.double.cp
//               ^^ constant.character.escape.cp

@memo
// <- punctuation.definition.decorator.cp
// <~---- meta.decorator.cp entity.name.function.decorator.cp

@brute(count_brute)
// <~----- entity.name.function.decorator.cp
//     ^^^^^^^^^^^ - entity.name.function

fn dijkstra(src: V) -> [V]?Int:
// <-- storage.type.function.cp
// ^^^^^^^^ entity.name.function.cp
//          ^^^ variable.parameter.label.cp
//               ^ entity.name.type.cp
//                  ^^ keyword.operator.arrow.cp
//                        ^ keyword.operator.optional.cp
//                         ^^^ support.type.builtin.cp

fn sq(x: Int) -> Int = x * x
// ^^ entity.name.function.cp
//    ^ variable.parameter.label.cp
//                   ^ keyword.operator.assignment.cp
//                       ^ keyword.operator.arithmetic.cp

pub fn link(mut g: Dsu, r: Road; strict: Bool = false):
// <--- storage.modifier.pub.cp
//     ^^^^ entity.name.function.cp
//          ^^^ storage.modifier.mut.cp
//              ^ variable.parameter.label.cp
//                 ^^^ entity.name.type.cp
//                      ^ variable.parameter.label.cp
//                               ^^^^^^ variable.parameter.label.cp
//                                       ^^^^ support.type.builtin.cp

@local fn dump(g: Graph<V>) = print(g)
// <~----- entity.name.function.decorator.cp
//     ^^ storage.type.function.cp
//        ^^^^ entity.name.function.cp
//                ^^^^^ entity.name.type.cp
//                     ^ punctuation.definition.generic.begin.cp
//                       ^ punctuation.definition.generic.end.cp
//                          ^ keyword.operator.assignment.cp

pure fn old() -> Int = 1
// <---- invalid.deprecated.pure.cp
//      ^^^ entity.name.function.cp

struct Edge(u: V, v: V, w: Int = 0):
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

const best = Monoid<Int>(id: -INF, op: max)
// <----- storage.modifier.const.cp
//    ^^^^ variable.other.constant.cp
//           ^^^^^^ entity.name.type.cp
//                       ^^ variable.parameter.label.cp
//                            ^^^ support.constant.inf.cp
//                                 ^^ variable.parameter.label.cp
//                                     ^^^ - support.function

const MOD = 998244353
//    ^^^ variable.other.constant.cp
//          ^^^^^^^^^ constant.numeric.integer.decimal.cp

let y = x % MOD
//          ^^^ constant.other.caps.cp

trait Monoid<T>:
// <----- storage.type.trait.cp
//    ^^^^^^ entity.name.type.trait.cp

    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
//  ^^^ storage.type.law.cp
//      ^^^^^ entity.name.function.law.cp
//            ^ variable.parameter.label.cp
//                                ^^ entity.name.function.call.cp

impl Ord for Edge:
// <---- storage.type.impl.cp
//   ^^^ entity.name.type.cp
//       ^^^ keyword.control.for.cp
//           ^^^^ entity.name.type.cp

protocol Guess:
// <-------- storage.type.protocol.cp
//       ^^^^^ entity.name.type.protocol.cp

    @wire("?") fn ask(x: 1..=1e9) -> Ordering
//   ^^^^ entity.name.function.decorator.cp
//                ^^^ entity.name.function.cp
//                        ^^^ keyword.operator.range.cp
//                                   ^^^^^^^^ entity.name.type.cp

static let cache = build()
// <------ storage.modifier.static.cp
//     ^^^ storage.type.let.cp

fn each(inline f: (V) -> Int) -> yields V:
//      ^^^^^^ storage.modifier.inline.cp
//             ^ variable.parameter.label.cp
//                               ^^^^^^ keyword.other.yields.cp

decorator timed(f):
// <--------- storage.type.decorator.cp
//        ^^^^^ entity.name.function.decorator.cp

spec fn sorted(xs: [I]Int) -> Bool
// <---- storage.type.spec.cp
//   ^^ storage.type.function.cp
//      ^^^^^^ entity.name.function.cp

spec:
// <---- storage.type.spec.cp

fn bsearch(lo: Int, hi: Int) -> Int:
// ^^^^^^^ entity.name.function.cp

    requires lo <= hi
//  ^^^^^^^^ keyword.other.contract.requires.cp

    ensures result >= 0
//  ^^^^^^^ keyword.other.contract.ensures.cp

    while l < r:
//  ^^^^^ keyword.control.while.cp

        invariant l <= r
//      ^^^^^^^^^ keyword.other.contract.invariant.cp

        invariant = l
//      ^^^^^^^^^ - keyword

@invariant(swap(x, y))
// <~--------- entity.name.function.decorator.cp
//         ^^^^ entity.name.function.call.cp

    defer pool.free(node)
//  ^^^^^ keyword.control.defer.cp
//             ^^^^ entity.name.function.method.cp
