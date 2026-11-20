// SYNTAX TEST "source.cp" "CP Lang v2.2 parity and zero stale v1/v2.1 tokens"

// Core v2.2 stdlib and builtin generic types
const P = 998_244_353
type Z = Mod<P>
//       ^^^ support.type.stdlib.cp
//          ^ punctuation.definition.generic.begin.cp
//           ^ entity.name.type.cp
//            ^ punctuation.definition.generic.end.cp

let comb = Comb<P>()
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

var rng = Rng(42)
//        ^^^ support.type.builtin.cp

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

// v2.2 constructs: domain, lends, lends mut, @law, invariant, prev_permutation, diverging ??, sol: Interactor
domain D = 1..=n
// <------ storage.type.domain.cp
//     ^ entity.name.type.domain.cp
//          ^^^ keyword.operator.range.cp

for d in D:
// <--- keyword.control.for.cp
//    ^^ keyword.control.in.cp
    invariant d.l <= d.r
//  ^^^^^^^^^ keyword.other.contract.invariant.cp
//              ^ variable.other.property.boundary.cp
//                     ^ variable.other.property.boundary.cp
    best min= cost[d] ?? break
//       ^^^^ keyword.operator.assignment.compound.cp
//                    ^^ keyword.operator.coalesce.cp
//                       ^^^^^ keyword.control.break.cp
    best max= gain[d]
//       ^^^^ keyword.operator.assignment.compound.cp

while prev_permutation(mut perm):
//    ^^^^^^^^^^^^^^^^ support.function.builtin.cp
//                     ^^^ storage.modifier.mut.cp
    let k = ceil_div(a, b) + rand(1..=10)
//          ^^^^^^^^ support.function.builtin.cp
//                           ^^^^ support.function.builtin.cp
    shuffle(mut perm)
//  ^^^^^^^ support.function.builtin.cp

fn slice(self, s: Range<D>) lends mut [D]Int:
//                          ^^^^^ storage.modifier.lends.cp
//                                ^^^ storage.modifier.mut.cp
    @law disjoint(s, t) = s.r <= t.l or t.r <= s.l
//  ^ punctuation.definition.decorator.cp
//   ^^^ entity.name.function.decorator.cp
//       ^^^^^^^^ entity.name.function.call.cp
//                          ^ variable.other.property.boundary.cp
//                                 ^ variable.other.property.boundary.cp

let vals = read(gen: () => gen_case())
//         ^^^^ support.function.builtin.cp
//              ^^^ variable.parameter.label.cp
assert all(x >= 0 for x in vals)
// <------ keyword.control.assert.cp
//     ^^^ support.function.builtin.cp

// v2.2 decorators + explicit sol: Interactor
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
fn judge(sol: Interactor):
//       ^^^ variable.parameter.label.cp
//            ^^^^^^^^^^ support.type.builtin.cp
    print("ans = {best}, avg = {avg:.6f}")
//  ^^^^^ support.function.builtin.cp
//                ^^^^ meta.interpolation.cp
//                                 ^ punctuation.separator.format.cp
//                                  ^^^ constant.other.placeholder.format.cp

// Removed v2.1 keywords must NOT receive keyword/storage scopes in v2.2
let removed_v21 = defer + assume + where + inline + lend + law
//                ^^^^^ - keyword
//                        ^^^^^^ - keyword
//                                 ^^^^^ - keyword
//                                         ^^^^^^ - storage
//                                                  ^^^^ - keyword
//                                                         ^^^ - storage

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
