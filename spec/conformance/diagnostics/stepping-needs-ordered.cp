// expect: E0203
// engines: next
// rule: 04.2.31
// R04.2.31: `Domain` counts positions and `Ordered` steps between them, so `.next` under a plain
// `D: Domain` bound is an error that names `Ordered`.
fn second<D: Domain>(d: D) -> ?D = d.next // <- here
