//! The op shapes the VM compiles a hot builtin or library path to, and
//! what the program prints.

use super::*;

#[test]
fn vm_widens_and_multiplies_i128_on_its_own_ops() {
    // std/geometry's `cross128`, run once per comparison by an angular sort:
    // `I128(x)`, `+`, `-` and `*` of `I128`s and an `I128` compare stay on VM
    // ops, with no `apply` operand list; overflow still traps.
    let source = "
fn cross(ax: Int, ay: Int, bx: Int, by: Int) -> I128 =
    I128(ax) * I128(by) - I128(ay) * I128(bx) + I128(1)
fn ccw(ax: Int, ay: Int, bx: Int, by: Int) -> Bool = cross(ax, ay, bx, by) > I128(0)
let big = 3_000_000_000_000_000_000
print(Int(cross(3, 1, 2, 5)), yes(ccw(1, 0, 0, 1)), yes(ccw(0, 1, 1, 0)))
print(cross(big, 0, 0, big) / I128(big))
var x = I128(big)
for i in 0..3: x = x * x
";
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    for (id, f) in program.fns.iter().enumerate() {
        if f.name != "cross" && f.name != "ccw" {
            continue;
        }
        let chunk = compiled.fns[id].as_ref().expect("runs on the VM");
        let ops: Vec<String> = chunk.ops.iter().map(|op| format!("{op:?}")).collect();
        let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
        assert_eq!(count("ApplyV"), 0, "{}: {ops:?}", f.name);
        if f.name == "cross" {
            assert_eq!(count("IntToFloat"), 5, "{ops:?}");
            assert_eq!(count("FloatBin"), 4, "{ops:?}");
        }
    }
    let (out, trap) = outcome(source, "");
    assert_eq!(out, "14 YES NO\n3000000000000000000\n");
    let trap = trap.expect("the third square overflows");
    assert!(trap.contains("does not fit in an `I128`"), "{trap}");
}

#[test]
fn keyed_sort_and_runs_call_the_key_once_per_element() {
    // cco16p2's `seen.sort(by: s => arg(s.dir))` then `seen.runs(by: ...)`:
    // each key runs once per element, equal keys keep their order, and a run
    // breaks where the user `Eq` says two neighbours' keys differ.
    let source = "
struct Half(v: Int)
impl Ord for Half:
    fn lt(a, b) = a.v / 2 < b.v / 2
impl Eq for Half:
    fn eq(a, b) = a.v / 2 == b.v / 2
fn key(x: (Int, Int)) -> Half:
    print(\"k\", x.1)
    Half(x.0)
var xs = [(3, 0), (1, 1), (2, 2), (0, 3), (3, 4), (5, 5)]
xs.sort(by: x => key(x))
print([x.1 for x in xs])
print([r.len for r in xs.runs(by: x => key(x))])
let none: List<(Int, Int)> = []
print(none.runs(by: x => key(x)).count())
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    let keys = |ids: [u8; 6]| ids.map(|i| format!("k {i}\n")).concat();
    let want =
        keys([0, 1, 2, 3, 4, 5]) + "1 3 0 2 4 5\n" + &keys([1, 3, 0, 2, 4, 5]) + "2 3 1\n0\n";
    assert_eq!(out, want);
}
