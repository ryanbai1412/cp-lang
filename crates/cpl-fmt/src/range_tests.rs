use super::*;
use crate::format;

fn apply(src: &str, edit: Option<&Edit>) -> String {
    match edit {
        None => src.to_string(),
        Some(e) => format!("{}{}{}", &src[..e.range.start], e.text, &src[e.range.end..]),
    }
}

/// `src` with `range` formatted; formatting the edited span again changes
/// nothing.
fn ranged(src: &str, range: Range<usize>) -> String {
    let style = Style::default();
    let edit = format_range(src, &style, range).unwrap_or_else(|e| panic!("{e}\n{src}"));
    let out = apply(src, edit.as_ref());
    if let Some(e) = &edit {
        let again = format_range(&out, &style, e.range.start..e.range.start + e.text.len());
        assert_eq!(
            again.unwrap(),
            None,
            "formatting the range twice changed it:\n{out}"
        );
    }
    out
}

fn typed(src: &str, ch: char) -> String {
    let edit = format_on_type(src, &Style::default(), src.len(), ch).unwrap();
    apply(src, edit.as_ref())
}

fn line(src: &str, n: usize) -> Range<usize> {
    let start: usize = src.split_inclusive('\n').take(n - 1).map(str::len).sum();
    start..start + src[start..].find('\n').unwrap_or(src.len() - start)
}

#[test]
fn formats_only_the_selected_statement() {
    let src = "let a=1\nlet b=2\nlet c=3\n";
    assert_eq!(ranged(src, line(src, 2)), "let a=1\nlet b = 2\nlet c=3\n");
    assert_eq!(ranged(src, 9..10), "let a=1\nlet b = 2\nlet c=3\n");
    assert_eq!(
        ranged(src, 0..src.len()),
        "let a = 1\nlet b = 2\nlet c = 3\n"
    );
}

#[test]
fn widens_to_the_whole_statement() {
    let src = "let a=1\nlet xs=[1,\n2]\nlet c=3\n";
    assert_eq!(
        ranged(src, line(src, 3)),
        "let a=1\nlet xs = [1, 2]\nlet c=3\n"
    );
}

#[test]
fn formats_a_statement_inside_a_block() {
    let src = "fn f():\n    let a=1\n    let b=2\nlet c=3\n";
    assert_eq!(
        ranged(src, line(src, 3)),
        "fn f():\n    let a=1\n    let b = 2\nlet c=3\n"
    );
}

#[test]
fn widens_to_the_item_when_its_block_is_indented_otherwise() {
    let src = "let z=0\nfn f():\n  let a=1\n  if a==1:\n    let b=2\nlet c=3\n";
    assert_eq!(
        ranged(src, line(src, 4)),
        "let z=0\nfn f():\n    let a = 1\n    if a == 1:\n        let b = 2\nlet c=3\n"
    );
}

#[test]
fn keeps_crlf_line_ends() {
    let src = "let a=1\r\nlet b=2\r\nlet c=3\r\n";
    assert_eq!(ranged(src, 9..16), "let a=1\r\nlet b = 2\r\nlet c=3\r\n");
}

#[test]
fn keeps_comments_and_blank_lines_outside_the_range() {
    let src = "let a=1\n\n\n// note\nlet b=2  // why\nlet c=3\n";
    assert_eq!(
        ranged(src, line(src, 5)),
        "let a=1\n\n\n// note\nlet b = 2  // why\nlet c=3\n"
    );
}

#[test]
fn a_newline_formats_the_line_above_and_leaves_the_new_one() {
    assert_eq!(typed("let a=1\nlet b=2\n", '\n'), "let a=1\nlet b = 2\n");
    assert_eq!(
        typed("let a=1\nif a==1:\n    ", '\n'),
        "let a=1\nif a == 1:\n    "
    );
    assert_eq!(
        typed("for i in 0..n:\n    s+=i\n    ", '\n'),
        "for i in 0..n:\n    s += i\n    "
    );
}

#[test]
fn a_colon_formats_its_line() {
    assert_eq!(typed("let a=1\nif a==1:", ':'), "let a=1\nif a == 1:");
}

#[test]
fn an_unfinished_statement_is_left_alone() {
    let style = Style::default();
    let src = "let xs = [1,\n";
    assert!(!matches!(
        format_on_type(src, &style, src.len(), '\n'),
        Ok(Some(_))
    ));
    let src = "let xs = f(1,\n    2,\n";
    assert!(!matches!(
        format_on_type(src, &style, src.len(), '\n'),
        Ok(Some(_))
    ));
}

/// Every line of the newcomer files and the lint records, formatted alone:
/// the file keeps its tokens, a second pass is a no-op, and the whole file
/// as a range is `format` (where no `use` run is sorted).
#[test]
fn every_line_of_the_newcomer_files_formats_alone() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files: Vec<_> = std::fs::read_dir(root.join("corpus/newcomer"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "cp"))
        .collect();
    files.sort();
    let mut lines = 0;
    for path in &files {
        let src = std::fs::read_to_string(path).unwrap();
        let Ok(whole) = format(&src) else { continue };
        if !src.contains("use ") {
            assert_eq!(ranged(&src, 0..src.len()), whole, "{}", path.display());
        }
        for n in 1..=src.lines().count() {
            let out = ranged(&src, line(&src, n));
            let tokens = cpl_syntax::lex(&src).tokens;
            verify(&src, &tokens, &out).unwrap_or_else(|e| panic!("{}:{n}: {e}", path.display()));
            lines += 1;
        }
    }
    assert!(lines > 500, "only {lines} lines formatted");
}

/// The edit an editor applies is the smallest one, and never splits a
/// character. Rule: `lints/R22`.
#[test]
fn an_edit_is_the_smallest_change_on_char_boundaries() {
    let span = |e: Option<Edit>| e.map(|e| (e.range, e.text));
    assert_eq!(span(edit("let b=2", 0..7, "let b=2")), None);
    assert_eq!(
        span(edit("let b=2", 0..7, "let b = 2")),
        Some((5..6, " = ".to_string()))
    );
    assert_eq!(
        span(edit("xxlet b=2", 2..9, "let b = 2")),
        Some((7..8, " = ".to_string()))
    );
    assert_eq!(
        span(edit("// é", 0..5, "// è")),
        Some((3..5, "è".to_string()))
    );
    assert_eq!(
        span(edit("// é", 0..5, "// ɩ")),
        Some((3..5, "ɩ".to_string()))
    );
}

/// A range in a CRLF file names the same lines it does with LF ends,
/// including a cursor between `\r` and `\n`.
#[test]
fn a_crlf_range_maps_to_its_own_lines() {
    let src = "let a=1\r\nlet b=2\r\nlet c=3\r\n";
    assert_eq!(ranged(src, 18..25), "let a=1\r\nlet b=2\r\nlet c = 3\r\n");
    assert_eq!(ranged(src, 8..8), "let a = 1\r\nlet b=2\r\nlet c=3\r\n");
}

/// A selected statement with a trailing comma formats as the whole file
/// would.
#[test]
fn a_range_with_a_trailing_comma_formats_like_the_file() {
    let src = "let a=1\nlet xs=[1,2,]\n";
    let want = format!("let a=1\n{}", format("let xs=[1,2,]\n").unwrap());
    assert_eq!(ranged(src, line(src, 2)), want);
}
