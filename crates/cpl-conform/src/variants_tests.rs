use super::*;

#[test]
fn probes_print_after_loops_and_write_each_var() {
    let text = "var a = 0\nfor i in 0..3:\n    a += i\nvar b = [1]\nprint(a)\n";
    let ps = probes(text);
    assert_eq!(
        ps[0],
        Probe {
            at: Some(3),
            lines: vec!["print(a)".into()]
        }
    );
    assert!(ps
        .iter()
        .filter(|p| p.at.is_none())
        .any(|p| p.lines[0] == "print(b)"));
    let keep = vec![true; ps.len()];
    let (out, owner) = apply(text, &ps, &keep);
    assert!(out.starts_with("var a = 0\nfor i in 0..3:\n    a += i\nprint(a)\nvar b = [1]\n"));
    assert_eq!(owner[3], Some(0));
    assert_eq!(owner[4], None);
}

#[test]
fn build_keeps_only_probes_that_check() {
    let mut db = Session::new();
    let text = "var n = 3\nvar xs = [1, 2, 3]\nfor i in 0..n:\n    xs.push(i)\nprint(xs.len)\n";
    let (variant, kept) = build(&mut db, "probe.cp", text).expect("some probe checks");
    assert!(kept > 2, "{variant}");
    assert!(variant.contains("cpl_probe0_bump(mut n)"), "{variant}");
    assert!(variant.contains("cpl_probe1_rev(mut xs)"), "{variant}");
    assert!(!variant.contains("cpl_probe1_bump"), "{variant}");
    assert!(error_lines(&mut db, "probe.cp", &variant).is_none());
}

#[test]
fn a_port_without_probe_sites_is_its_own_variant() {
    let flat = "let n = 3\nprint(n + 1)\n";
    assert!(super::same_as_port(flat));
    let looped = "var s = 0\nfor i in 0..3:\n    s += i\nprint(s)\n";
    assert!(!super::same_as_port(looped));
}

#[test]
fn a_var_that_is_not_show_prints_through_yes_or_a_default() {
    let mut db = Session::new();
    let text = "var seen: [0..3]Bool = false\nvar best: [0..3]?Int = None\nfor i in 0..3:\n    seen[i] = true\n    best[i] = Some(i)\nprint(seen.count(b => b))\n";
    let (variant, _) = build(&mut db, "probe.cp", text).expect("some probe checks");
    assert!(
        variant.contains("print([yes(cpl_x) for cpl_x in seen])"),
        "{variant}"
    );
    assert!(
        variant.contains("print([cpl_x ?? -1 for cpl_x in best])"),
        "{variant}"
    );
    assert!(error_lines(&mut db, "probe.cp", &variant).is_none());
}

#[test]
fn a_hash_container_prints_in_an_order_both_engines_share() {
    let mut db = Session::new();
    let text = "var seen: HashMap<Int, Int> = [3: 1]\nvar got: HashSet<Int> = HashSet()\nfor i in 0..3:\n    seen[i] = i\n    got.add(i)\nprint(seen.len, got.len)\n";
    let (variant, _) = build(&mut db, "probe.cp", text).expect("some probe checks");
    assert!(!variant.contains("print(seen)\n"), "{variant}");
    assert!(!variant.contains("print(got)\n"), "{variant}");
    assert!(variant.contains("sorted()"), "{variant}");
    assert!(error_lines(&mut db, "probe.cp", &variant).is_none());
}

#[test]
fn a_hash_container_inside_a_table_prints_its_sizes() {
    let mut db = Session::new();
    let text = "domain R = 0..3\nvar cnt: [R]HashMap<Int, Int> = HashMap()\nfor r in R:\n    cnt[r][Int(r)] = 1\nprint(cnt[R(0)].len)\n";
    let (variant, _) = build(&mut db, "probe.cp", text).expect("some probe checks");
    assert!(variant.contains(".len for cpl_x in cnt]"), "{variant}");
    assert!(error_lines(&mut db, "probe.cp", &variant).is_none());
}
