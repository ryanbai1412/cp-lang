// SYNTAX TEST "source.cp" "CP Lang v2.1 types, domains, borrows and escape hatches"

let (n, m): (1..=2e5, 1..=2e5) = read()
// <--- storage.type.let.cp
//           ^ constant.numeric.integer.decimal.cp
//            ^^^ keyword.operator.range.cp
//               ^^^ constant.numeric.integer.decimal.cp
//                               ^^^^ support.function.builtin.cp

domain V = 1..=n
// <------ storage.type.domain.cp
//     ^ entity.name.type.domain.cp
//          ^^^ keyword.operator.range.cp

domain Node = In + Out
//     ^^^^ entity.name.type.domain.cp
//            ^^ entity.name.type.cp
//               ^ keyword.operator.arithmetic.cp
//                 ^^^ entity.name.type.cp

let edges: [E](u: V, v: V, w: 1..=1e9) = read()
//         ^ punctuation.section.brackets.begin.cp
//          ^ entity.name.type.cp
//             ^ variable.parameter.label.cp
//                         ^ variable.parameter.label.cp
//                                ^^^ constant.numeric.integer.decimal.cp

var dist: [V]?Int = none
// <--- storage.type.var.cp
//         ^ entity.name.type.cp
//           ^ keyword.operator.optional.cp
//            ^^^ support.type.builtin.cp
//                  ^^^^ constant.language.none.cp

var grid: [Row, Col]Char = '.'
//         ^^^ entity.name.type.cp
//              ^^^ entity.name.type.cp
//                  ^^^^ support.type.builtin.cp
//                         ^^^ string.quoted.single.char.cp

var pre: [V.bounds]Int = 0
//        ^ entity.name.type.cp
//          ^^^^^^ variable.other.property.cp

var pq = MinHeap<(Int, V)>()
//       ^^^^^^^ support.type.builtin.cp
//              ^ punctuation.definition.generic.begin.cp
//                ^^^ support.type.builtin.cp
//                       ^ punctuation.definition.generic.end.cp

var g: [V]List<(V, Int)> = []
//        ^^^^ support.type.builtin.cp
//                     ^ punctuation.definition.generic.end.cp
//                       ^ keyword.operator.assignment.cp

var m: Map<Str, List<Set<U64>>> = Map()
//     ^^^ support.type.builtin.cp
//         ^^^ support.type.builtin.cp
//                   ^^^ support.type.builtin.cp
//                       ^^^ support.type.builtin.cp
//                          ^^^ punctuation.definition.generic.end.cp
//                              ^ keyword.operator.assignment.cp

// A capitalized name and `<` with a space between is a comparison.
if PI < x and N > 3:
// ^^ support.constant.math.cp
//    ^ keyword.operator.comparison.cp
//            ^ entity.name.type.cp
//              ^ keyword.operator.comparison.cp

let w: F64 = F64(x) + F64(I128.MAX)
//     ^^^ support.type.builtin.cp
//           ^^^ support.type.builtin.cp
//                        ^^^^ support.type.builtin.cp
//                             ^^^ constant.other.caps.cp

let twice = [V] u => deg[u] * 2
//           ^ entity.name.type.cp
//              ^ variable.parameter.lambda.cp
//                ^^ keyword.operator.fat-arrow.cp

sort(mut xs, by: (a, b) => a.w < b.w)
// <---- support.function.builtin.cp
//   ^^^ storage.modifier.mut.cp
//           ^^ variable.parameter.label.cp
//                      ^^ keyword.operator.fat-arrow.cp
//                           ^ variable.other.property.cp

fn link(mut g: [V]List<V>, r: Road):
//      ^^^ storage.modifier.mut.cp
//          ^ variable.parameter.label.cp

link(mut g, roads[e])
// <---- entity.name.function.call.cp
//   ^^^ storage.modifier.mut.cp

// A projection is declared with `lends` / `lends mut`.
    fn at(self, i: I) lends Elem
//     ^^ entity.name.function.cp
//                    ^^^^^ storage.modifier.lends.cp
//                          ^^^^ entity.name.type.cp

    fn at(self, s: Range<I>) lends mut Slice<I, T>:
//                           ^^^^^ storage.modifier.lends.cp
//                                 ^^^ storage.modifier.mut.cp
//                                     ^^^^^ entity.name.type.cp

        law disjoint(s, t) = s.hi <= t.lo or t.hi <= s.lo
//      ^^^ storage.type.law.cp
//          ^^^^^^^^ entity.name.function.law.cp
//                         ^ keyword.operator.assignment.cp
//                                ^^ keyword.operator.comparison.cp
//                                        ^^ keyword.operator.logical.or.cp

    pub fn pair_mut(mut self, i: I, j: I) lends (mut T, mut T):
//         ^^^^^^^^ entity.name.function.cp
//                                        ^^^^^ storage.modifier.lends.cp
//                                                      ^^^ storage.modifier.mut.cp

    pub fn row(self, r: A) lends mut [B]T:
//         ^^^ entity.name.function.cp
//                         ^^^^^ storage.modifier.lends.cp
//                               ^^^ storage.modifier.mut.cp
//                                    ^ entity.name.type.cp

fn row(self, r: A, inline use: ([B]T) -> R) -> R =
//                 ^^^^^^ storage.modifier.inline.cp
//                        ^^^ variable.parameter.label.cp
//                        ^^^ - keyword
//                                    ^^ keyword.operator.arrow.cp

    pub fn table(inline f: I -> T) -> Array<I, T> = build(f)
//               ^^^^^^ storage.modifier.inline.cp
//                      ^ variable.parameter.label.cp
//                                    ^^^^^ entity.name.type.cp

swap(mut g.row(r1), mut g.row(r2))
// <---- support.function.builtin.cp
//                  ^^^ storage.modifier.mut.cp
//         ^^^ entity.name.function.method.cp

impl Index for V:
// <---- storage.type.impl.cp
//   ^^^^^ entity.name.type.cp
//         ^^^ keyword.control.for.cp

    let K = n
//  ^^^ storage.type.let.cp
//      ^ entity.name.type.cp

    type bounds = Bounds<Self>
//  ^^^^ storage.type.alias.cp
//       ^^^^^^ entity.name.type.alias.cp
//                ^^^^^^ support.type.builtin.cp
//                       ^^^^ entity.name.type.cp

trait Seq: Index + Ord
//    ^^^ entity.name.type.trait.cp
//                 ^^^ entity.name.type.cp

    @get fn l(self) -> Bounds<Self> = Bounds.from_idx(self.to_idx)
//   ^^^ entity.name.function.decorator.cp
//          ^ entity.name.function.cp
//                                           ^^^^^^^^ entity.name.function.method.cp
//                                                         ^^^^^^ variable.other.property.cp

pub struct Bounds<I>(ord: 0..=I.K) where I: Seq:
//         ^^^^^^ entity.name.type.struct.cp
//                   ^^^ variable.parameter.label.cp
//                              ^ entity.name.type.cp
//                                 ^^^^^ keyword.control.where.cp

fn longest_true<A>(a: A) -> Int where A: Indexed<V, Elem = Bool>
// ^^^^^^^^^^^^ entity.name.function.cp
//             ^ punctuation.definition.generic.begin.cp
//              ^ entity.name.type.cp
//               ^ punctuation.definition.generic.end.cp
//                 ^ variable.parameter.label.cp
//                              ^^^^^ keyword.control.where.cp
//                                                         ^^^^ support.type.builtin.cp

pub fn compress_vals(xs: List<Int>) -> (domain X, vals: [X]Int, rank: List<X>):
//                                      ^^^^^^ storage.type.domain.cp
//                                             ^ entity.name.type.domain.cp
//                                                ^^^^ variable.parameter.label.cp
//                                                              ^^^^ variable.parameter.label.cp

domain L = 'A'..'A' + k
//         ^^^ string.quoted.single.char.cp
//            ^^ keyword.operator.range.cp

// `.l` and `.r` are the boundaries left and right of a position; `.l(k)` and `.r(k)` step k further.
for v in V: pre[v.r] = pre[v.l] + a[v]
//                ^ variable.other.property.boundary.cp
//                           ^ variable.other.property.boundary.cp

let window = pre[v.r] - pre[v.l(k)] + pre[v.r(k)]
//                 ^ variable.other.property.boundary.cp
//                            ^ variable.other.property.boundary.cp
//                                          ^ variable.other.property.boundary.cp
//                                          ^ - entity.name.function

let width = s.r - s.l
//            ^ variable.other.property.boundary.cp
//                  ^ variable.other.property.boundary.cp

let mid = s.len / 2 + s.lo
//          ^^^ variable.other.property.cp
//                      ^^ variable.other.property.cp

// Escape hatches carry the `unsafe` scope themes already colour.
let x = a.raw[i + 1]
//        ^^^ keyword.other.unsafe.raw.cp
//              ^ keyword.operator.arithmetic.cp

unchecked:
// <--------- keyword.other.unsafe.unchecked.cp
//       ^ punctuation.separator.colon.cp

    total += a.raw[i] * b.raw[i]
//             ^^^ keyword.other.unsafe.raw.cp
//                        ^^^ keyword.other.unsafe.raw.cp

let raw = rows.len
//  ^^^ - keyword

let reqs: [q](l: I, r: I, x: X) = read() where all(t.l <= t.r for t in reqs)
//            ^ variable.parameter.label.cp
//                                       ^^^^^ keyword.control.where.cp
//                                             ^^^ support.function.builtin.cp
//                                                            ^^^ keyword.control.for.cp
