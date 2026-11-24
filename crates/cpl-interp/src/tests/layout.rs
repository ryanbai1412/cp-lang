//! How values sit in VM registers and packed lists, and that they change in place.

use super::*;

#[test]
fn vm_frames_hold_their_widest_statement_not_every_temporary() {
    // 600 locals and 42,000 statements: more temporaries than a register
    // index addresses, unless each statement's are freed when it ends.
    let mut source = String::from("fn wide(x: Int) -> Int:\n");
    for i in 0..600 {
        source += &format!("    let a{i} = x + {i}\n");
    }
    source += "    var s = 0\n";
    for _ in 0..70 {
        for i in 0..600 {
            source += &format!("    s += a{i} * 1\n");
        }
    }
    source += "    s\nprint(wide(read()))\n";
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source.as_str());
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let wide = compiled.fns[0]
        .as_ref()
        .expect("a 600-local fn runs on the VM");
    assert!(wide.num_i < 1000, "{} int registers", wide.num_i);
    assert_eq!(outcome(&source, "3"), ("12705000\n".to_string(), None));
}

#[test]
fn vm_rolling_array_move_semantics() {
    let source = "
let n = read(1..=20)
domain I = 0..n
var ways: [I]Int = 0
ways[I.first!] = 1
for step in 0..n:
    var next: [I]Int = 0
    for i in I:
        next[i] = ways[i] + (if let p = i.prev: ways[p] else: 0)
    ways = next
    ways[I.first!] += 1
print(ways[I.last!])
";
    let (out, trap) = outcome(source, "5\n");
    assert_eq!(trap, None);
    assert_eq!(out, "6\n");
}

#[test]
fn vm_keeps_a_heap_of_int_pairs_in_int_registers() {
    // dijkstra's `MinHeap<(W, V)>`: entries move between the list and
    // registers, and `less` is inlined into `push` and `sift_down`, so only
    // a `by:` heap's comparator closure takes boxed entries.
    let source = "
fn order(xs: List<(Int, Int)>) -> Int:
    var h = MinHeap<(Int, Int)>()
    for x in xs: h.push(x)
    var acc = 0
    while let (d, v) = h.pop():
        acc = acc * 10 + v + d - d
    acc
fn by_second(xs: List<(Int, Int)>) -> Int:
    var h = Heap<(Int, Int)>.new(by: p => -p.1)
    for x in xs: h.push(x)
    var acc = 0
    while let (d, v) = h.pop():
        acc = acc * 10 + d
    acc
let xs = [(3, 1), (1, 2), (2, 3), (1, 0), (5, 4), (2, 5)]
print(order(xs))
print(by_second(xs))
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
    let mut seen = 0;
    for (id, f) in program.fns.iter().enumerate() {
        if !f.name.starts_with("Heap_")
            || ![".push", ".sift_down"].iter().any(|h| f.name.ends_with(h))
        {
            continue;
        }
        seen += 1;
        let chunk = compiled.fns[id].as_ref().expect("heap fns run on the VM");
        let ops: Vec<String> = chunk.ops.iter().map(|op| format!("{op:?}")).collect();
        let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
        assert_eq!(count("CallVmMix"), 0, "{} calls `less`: {ops:?}", f.name);
        assert_eq!(
            count("UnboxSmallTuple"),
            0,
            "{} unboxes an entry: {ops:?}",
            f.name
        );
        assert!(count("CmpSmallTuple") > 0, "{ops:?}");
        // The comparator closure takes its entries in Int registers too.
        assert_eq!(
            count("BoxSmallTuple"),
            0,
            "{} boxes an entry: {ops:?}",
            f.name
        );
    }
    assert!(seen >= 2, "{seen}");
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "23514\n252131\n");
}

#[test]
fn vm_builds_wide_int_structs_from_int_registers() {
    // bench/ranges/lazy_assign's `join`: a 5-field struct of bounded Ints is
    // built straight from Int registers, each field checked against constant bounds,
    // and returned in them: no tuple is boxed.
    let source = "
struct Run(a: 0..=100, b: 0..=100, c: 0..=100, d: 0..=100, e: 0..=100)
fn join(x: Run, y: Run) -> Run = Run(x.a + y.a, x.b + y.b, x.c, y.d, x.e + 1)
let r = join(Run(1, 2, 3, 4, 5), Run(10, 20, 30, 40, 50))
print(r.a, r.b, r.c, r.d, r.e)
let big = join(Run(99, 0, 0, 0, 0), Run(2, 0, 0, 0, 0))
print(big.a)
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
    let join = program.fns.iter().position(|f| f.name == "join").unwrap();
    let chunk = compiled.fns[join].as_ref().expect("join is on the VM");
    let ops: Vec<String> = chunk.ops.iter().map(|op| format!("{op:?}")).collect();
    let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
    assert_eq!(count("MakeTupleI"), 0, "{ops:?}");
    assert_eq!(count("RetSmallTuple"), 1, "{ops:?}");
    assert_eq!(count("RangeCheckImmI"), 3, "{ops:?}");
    assert_eq!(count("BoxI"), 0, "{ops:?}");
    assert_eq!(count("ConstImmI"), 0, "{ops:?}");
    let (out, trap) = outcome(source, "");
    assert_eq!(out, "11 22 3 40 6\n");
    let msg = trap.expect("expected out-of-range trap");
    assert!(msg.contains("101 is outside"), "unexpected trap: {msg}");
}

#[test]
fn vm_holds_a_one_u64_struct_as_its_bits() {
    // atcoder_dp_u's `Subset<R>`: a struct of one `U64` is a scalar, passed
    // and returned in an Int register, its high bit kept.
    let source = "
struct W(b: U64)
fn sub(x: W, y: W) -> W = W(x.b & (y.b ^ (U64(0) - U64(1))))
let all = W(U64(0) - U64(1))
let r = sub(all, W(U64(6)))
print(r.b, yes(r == all), yes(sub(r, r) == W(U64(0))))
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
    let sub = program.fns.iter().position(|f| f.name == "sub").unwrap();
    let chunk = compiled.fns[sub].as_ref().expect("sub is on the VM");
    let held = format!("{:?}", &chunk.slot_map[..2]);
    assert_eq!(held.matches("OneU64Struct").count(), 2, "{held}");
    let (out, trap) = outcome(source, "");
    assert!(trap.is_none(), "{trap:?}");
    assert_eq!(out, "18446744073709551609 NO YES\n");
}

#[test]
fn vm_temp_v_register_drop_avoids_cow_cloning_on_global_list() {
    let source = "
var stk: List<(Int, Int)> = []
var acc = 0
for i in 0..2000:
    stk.push((i, i + 1))
    if let (a, b) = stk.last:
        acc += a + b
    stk.pop()
print(acc, stk.len)
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "4000000 0\n");
}

#[test]
fn a_value_is_three_words() {
    // Every list cell, register and stack slot is a Value; at 48 bytes a
    // List<Int> cost six times the C++ vector<long long>.
    assert_eq!(std::mem::size_of::<crate::value::Value>(), 24);
}

#[test]
fn packed_cells_move_like_a_vec_of_their_values() {
    use crate::value::{Cells, OptVal, Value};

    let kinds: [fn(i64) -> Value; 4] = [
        |i| Value::Bool(i % 2 == 0),
        |i| Value::Char(b'a' + i as u8),
        |i| {
            if i % 3 == 0 {
                Value::Opt(OptVal::None)
            } else {
                Value::some(Value::Int(i))
            }
        },
        |i| Value::Int(i * 7),
    ];
    for make in kinds {
        let mut want: Vec<Value> = (0..6).map(make).collect();
        let mut cells = Cells::from(want.clone());
        cells.insert(1, make(9));
        want.insert(1, make(9));
        cells.insert(cells.len(), make(10));
        want.push(make(10));
        assert_eq!(cells.remove(2), want.remove(2));
        assert_eq!(cells.remove(0), want.remove(0));
        cells.reverse();
        want.reverse();
        assert_eq!(cells.to_values(), want);
        assert!(!matches!(cells, Cells::Values(_)));
    }
}

#[test]
fn a_list_of_small_enums_is_packed() {
    use crate::value::{Cells, Value};
    use std::rc::Rc;
    let set = |i, b| Value::Enum(1, Rc::new(vec![Value::Int(i), Value::Bool(b)]));
    let mut asks = Cells::default();
    asks.push(set(3, true));
    asks.push(Value::Enum(0, Rc::new(Vec::new())));
    assert!(matches!(asks, Cells::Rows { stride: 4, .. }));
    assert_eq!(asks.get(0), Some(set(3, true)));
    asks.set(1, set(-4, false));
    assert_eq!(asks.pop(), Some(set(-4, false)));
    // A field that is not a scalar keeps every value, boxed.
    asks.push(Value::Enum(
        2,
        Rc::new(vec![Value::Str(Rc::new(b"x".to_vec()))]),
    ));
    assert!(matches!(asks, Cells::Values(_)));
    assert_eq!(asks.get(0), Some(set(3, true)));
}

#[test]
fn a_live_input_lets_go_of_what_it_read() {
    let lines: Vec<u8> = (0..100_000)
        .flat_map(|i| format!("{i}\n").into_bytes())
        .collect();
    let total = lines.len();
    let mut chunks = lines
        .chunks(4096)
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>()
        .into_iter();
    let mut input = crate::input::Input::live(Box::new(move || chunks.next()));
    for i in 0..100_000 {
        assert_eq!(input.int().ok().map(|(v, _)| v), Some(i));
    }
    assert!(input.bytes().len() < total / 4, "{}", input.bytes().len());
    let message = input.int().expect_err("ended").message;
    assert!(message.contains("input line 100001, column 1"), "{message}");
    let kept = (0..100)
        .flat_map(|i| format!("{i}\n").into_bytes())
        .collect::<Vec<_>>();
    let mut chunks = kept
        .chunks(7)
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>()
        .into_iter();
    let mut input = crate::input::Input::live(Box::new(move || chunks.next())).keeping(true);
    while input.int().is_ok() {}
    assert_eq!(input.bytes(), &kept[..]);
}

#[test]
fn a_list_of_scalars_is_packed() {
    use crate::value::{Cells, TupleVal, Value};
    // A List<Int> cell is 8 bytes and a list of (Int, Int) 16, as in the
    // C++, not a 24-byte Value (or a boxed tuple) each.
    let ints = Cells::from((0..4).map(Value::Int).collect::<Vec<_>>());
    assert!(matches!(ints, Cells::Int(_)));
    let mut pairs = Cells::default();
    for i in 0..3 {
        pairs.push(Value::tuple(vec![Value::Int(i), Value::Int(-i)]));
    }
    assert!(matches!(pairs, Cells::Rows { stride: 2, .. }));
    assert_eq!(
        pairs.pop(),
        Some(Value::tuple(vec![Value::Int(2), Value::Int(-2)]))
    );
    // A cell of another kind keeps every value, boxed.
    pairs.push(Value::Tuple(TupleVal::OneInt(7)));
    assert!(matches!(pairs, Cells::Values(_)));
    assert_eq!(pairs.len(), 3);
}

#[test]
fn a_packed_list_sorts_packed() {
    use crate::value::{Cells, Value};
    let ints = Cells::from([3, 1, 2].map(Value::Int).to_vec());
    let up = crate::sort::sorted_packed(&ints, &cpl_ir::Ty::Int, false).unwrap();
    assert!(matches!(&up, Cells::Int(v) if v == &[1, 2, 3]));
    let down = crate::sort::sorted_packed(&ints, &cpl_ir::Ty::Int, true).unwrap();
    assert!(matches!(&down, Cells::Int(v) if v == &[3, 2, 1]));
    // A type with an order of its own sorts through it.
    let opt = cpl_ir::Ty::Opt(Box::new(cpl_ir::Ty::Int));
    assert!(crate::sort::sorted_packed(&ints, &opt, false).is_none());
}

#[test]
fn a_table_of_optional_nested_tuples_is_packed() {
    use crate::value::{Cells, OptVal, TupleVal, Value};

    let tuple = |fs: Vec<Value>| Value::Tuple(TupleVal::from_vec(fs));
    let back = |v: i64, w: i64| {
        Value::some(tuple(vec![
            Value::Int(v),
            tuple(vec![Value::Int(v + 1), Value::Bool(w > 0), Value::Int(w)]),
        ]))
    };
    let mut cells = Cells::filled(4, Value::Opt(OptVal::None));
    cells.set(2, back(5, -7));
    assert!(matches!(cells, Cells::Rows { stride: 5, .. }));
    assert_eq!(cells.get(0), Some(Value::Opt(OptVal::None)));
    assert_eq!(cells.get(2), Some(back(5, -7)));
    cells.push(back(1, 3));
    cells.sort_unstable();
    let want = vec![
        Value::Opt(OptVal::None),
        Value::Opt(OptVal::None),
        Value::Opt(OptVal::None),
        back(1, 3),
        back(5, -7),
    ];
    assert_eq!(cells.to_values(), want);
    cells.set(0, Value::some(Value::Int(1)));
    assert!(matches!(cells, Cells::Values(_)));
    assert_eq!(cells.get(4), Some(back(5, -7)));
}

#[test]
fn operands_fields_and_variables_read_in_place() {
    let src = "\
let g = [4, 5, 6]
var m: Map<Int, Int> = []
m[2] = 7
let p = (3, 9)
let q = (8,)
print(g[1], p.1, q.0, -p.0, yes(2 in m), yes(5 in m), yes(p.0 < p.1))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "5 9 8 -3 YES NO YES\n");
}

#[test]
fn one_int_tuples_options_and_formats_change_in_place() {
    let src = "\
var t = (5,)
t.0 += 2
var m: ?Int = 3
m! += 4
let x = 1.5
print(t.0, m!, \"{t.0}:{x}\")
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "7 7 7:1.5000000000\n");
}
