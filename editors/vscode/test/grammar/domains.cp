// SYNTAX TEST "source.cp" "types, domains, borrows and escape hatches"

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

var pre: [I.bounds]Int = 0
//        ^ entity.name.type.cp
//          ^^^^^^ variable.other.property.cp

var pq = Heap<(Int, V)>(by: min)
//       ^^^^ entity.name.type.cp
//           ^ punctuation.definition.generic.begin.cp
//             ^^^ support.type.builtin.cp
//                    ^ punctuation.definition.generic.end.cp
//                      ^^ variable.parameter.label.cp
//                          ^^^ - support.function

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
if INF < x and N > 3:
// ^^^ support.constant.inf.cp
//     ^ keyword.operator.comparison.cp
//             ^ entity.name.type.cp
//               ^ keyword.operator.comparison.cp

let w: I32 = x.raw + I128.MAX
//     ^^^ support.type.builtin.cp
//                   ^^^^ support.type.builtin.cp
//                        ^^^ constant.other.caps.cp

let twice = [V] u => deg[u] * 2
//           ^ entity.name.type.cp
//              ^ variable.parameter.lambda.cp
//                ^^ keyword.operator.fat-arrow.cp

sort(xs, by: (a, b) => a.w < b.w)
// <---- entity.name.function.call.cp
//       ^^ variable.parameter.label.cp
//                  ^^ keyword.operator.fat-arrow.cp
//                       ^ variable.other.property.cp

fn link(mut g: [V]List<V>, r: Road):
//      ^^^ storage.modifier.mut.cp
//          ^ variable.parameter.label.cp

link(mut g, roads[e])
// <---- entity.name.function.call.cp
//   ^^^ storage.modifier.mut.cp

// Projections lend a part of their receiver to the caller.
fn left(self) -> lends [I]Int:
//               ^^^^^ storage.modifier.lends.cp
//                        ^^^ support.type.builtin.cp

fn halves(mut self, k: I) -> lends mut ([I]Int, [I]Int):
//                           ^^^^^ storage.modifier.lends.cp
//                                 ^^^ storage.modifier.mut.cp

    law disjoint(self.l(k), self.r(k))
//  ^^^ storage.type.law.cp
//      ^^^^^^^^ entity.name.function.law.cp
//                    ^ variable.other.property.boundary.cp
//                               ^ variable.other.property.boundary.cp
//                      ^ - variable.other.property

let width = s.r - s.l
//            ^ variable.other.property.boundary.cp
//                  ^ variable.other.property.boundary.cp

let mid = s.len / 2 + s.lo
//          ^^^ variable.other.property.cp
//                      ^^ variable.other.property.cp

// The escape hatches stand out: they carry the `unsafe` scope themes already colour.
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

pre[i.after] = pre[i.before] + a[i]
//    ^^^^^ variable.other.property.cp
//                   ^^^^^^ variable.other.property.cp
