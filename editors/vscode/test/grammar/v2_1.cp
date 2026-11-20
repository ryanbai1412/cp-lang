// SYNTAX TEST "source.cp" "CP Lang v2.1 parity and zero stale v1 tokens"

// Every v2.1 stdlib and builtin generic type
const P = 998_244_353
type Z = Mod<P>
//       ^^^ support.type.stdlib.cp
//          ^ punctuation.definition.generic.begin.cp
//           ^ entity.name.type.cp
//            ^ punctuation.definition.generic.end.cp

var comb = Comb<P>()
//         ^^^^ support.type.stdlib.cp
//             ^ punctuation.definition.generic.begin.cp
//               ^ punctuation.definition.generic.end.cp

var seen = Marks<D>()
//         ^^^^^ support.type.stdlib.cp
//              ^ punctuation.definition.generic.begin.cp
//                ^ punctuation.definition.generic.end.cp

let xs: Compressed<Int> = compress(raw_vals)
//      ^^^^^^^^^^ support.type.stdlib.cp
//                 ^^^ support.type.builtin.cp
//                        ^^^^^^^^ support.function.builtin.cp

var dq = MonoDeque<Int>(min)
//       ^^^^^^^^^ support.type.stdlib.cp
//                 ^^^ support.type.builtin.cp

var bs = Bitset<D>()
//       ^^^^^^ support.type.builtin.cp
//             ^ punctuation.definition.generic.begin.cp
//               ^ punctuation.definition.generic.end.cp

var st = SegTree<V, Int>(+, a)
//       ^^^^^^^ support.type.stdlib.cp
//                  ^^^ support.type.builtin.cp

var lazy = LazySegTree<t.Pre, Int>(AddSum, init)
//         ^^^^^^^^^^^ support.type.stdlib.cp
//                            ^^^ support.type.builtin.cp
//                                 ^^^^^^ support.type.stdlib.cp

let rmq = SparseTable<V, Int>(min, a)
//        ^^^^^^^^^^^ support.type.stdlib.cp
//                       ^^^ support.type.builtin.cp

var dsu = Dsu<V>()
//        ^^^ support.type.stdlib.cp
//           ^ punctuation.definition.generic.begin.cp
//             ^ punctuation.definition.generic.end.cp

let tree = Rooted<V>(g, V.first)
//         ^^^^^^ support.type.stdlib.cp
//               ^ punctuation.definition.generic.begin.cp
//                 ^ punctuation.definition.generic.end.cp

// v2.1 constructs: domain, lends, lends mut, for mut, defer, law, invariant, assume
domain D = 1..=n
// <------ storage.type.domain.cp
//     ^ entity.name.type.domain.cp
//          ^^^ keyword.operator.range.cp

for mut b in blocks:
// <--- keyword.control.for.cp
//  ^^^ storage.modifier.mut.cp
    defer dsu.undo_to(snap)
//  ^^^^^ keyword.control.defer.cp
    invariant b.l <= b.r
//  ^^^^^^^^^ keyword.other.contract.invariant.cp
//              ^ variable.other.property.boundary.cp
//                     ^ variable.other.property.boundary.cp
    best min= b.cost ?? 0
//       ^^^^ keyword.operator.assignment.compound.cp
//                   ^^ keyword.operator.coalesce.cp
    best max= b.gain
//       ^^^^ keyword.operator.assignment.compound.cp

fn slice(self, s: Range<D>) lends mut [D]Int:
//                          ^^^^^ storage.modifier.lends.cp
//                                ^^^ storage.modifier.mut.cp
    law disjoint(s, t) = s.r <= t.l or t.r <= s.l
//  ^^^ storage.type.law.cp
//      ^^^^^^^^ entity.name.function.law.cp
//                         ^ variable.other.property.boundary.cp
//                                ^ variable.other.property.boundary.cp

let vals = read(gen: () => gen_case())
//         ^^^^ support.function.builtin.cp
//              ^^^ variable.parameter.label.cp
assume all(x >= 0 for x in vals)
// <------ keyword.control.assume.cp
//     ^^^ support.function.builtin.cp

// v2.1 decorators
@memo
// <~---- entity.name.function.decorator.cp
@grader
// <~------ entity.name.function.decorator.cp
@multitest
// <~--------- entity.name.function.decorator.cp
@brute(main)
// <~----- entity.name.function.decorator.cp
@checker
// <~------- entity.name.function.decorator.cp
@interactor
// <~---------- entity.name.function.decorator.cp
fn demo():
    print("ans = {best}, avg = {avg:.6f}")
//  ^^^^^ support.function.builtin.cp
//                ^^^^ meta.interpolation.cp
//                                 ^ punctuation.separator.format.cp
//                                  ^^^ constant.other.placeholder.format.cp

// Stale v1 tokens must NOT receive keyword/builtin/constant scopes
let stale_types = (Float, I32, U8, ModR, INF)
//                 ^^^^^ - support.type
//                        ^^^ - support.type
//                             ^^ - support.type
//                                 ^^^^ - support.type
//                                       ^^^ - support.constant

let stale_words = by + with + static + protocol + decorator + do + yields + requires + ensures + pure
//                ^^ - keyword
//                     ^^^^ - keyword
//                            ^^^^^^ - storage
//                                     ^^^^^^^^ - storage
//                                                ^^^^^^^^^ - storage
//                                                            ^^ - keyword
//                                                                 ^^^^^^ - keyword
//                                                                          ^^^^^^^^ - keyword
//                                                                                     ^^^^^^^ - keyword
//                                                                                               ^^^^ - invalid
