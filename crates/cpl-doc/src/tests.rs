use super::*;

const LIB: &str = "\
/// Prefix sums over a list.
///
/// Build once in O(n), then answer any range sum in O(1).

/// Sums of every prefix of a list.
///
/// ```
/// let p = Prefix.of([3, 1, 4])
/// print(p.sum(0, 2))
/// ```
struct Prefix:
    /// `pre[i]` is the sum of the first `i` values.
    pre: List<Int>

    /// Builds the sums of `a`.
    @cost(\"O(n)\")
    fn of(a: List<Int>) -> Prefix:
        Prefix([0])

    /// The sum of `a[l..r]`. Both ends are positions.
    @cost(O(1))
    fn sum(self, l: Int, r: Int) -> Int = self.pre[r] - self.pre[l]

/// How a range is closed.
enum Bound:
    /// Includes its end.
    Closed
    Open(Int)

/// The largest value this library handles.
const LIMIT = 1e18
domain Small = 0..10
type Pair = (Int, Int)

fn _helper(x: Int) -> Int = x

/// The larger of two numbers.
fn larger<T>(
    a: T,
    b: T,
) -> T:
    if a < b: b else: a
";

fn lib() -> Module {
    extract(LIB, "lib/prefix.cp")
}

fn find<'a>(module: &'a Module, name: &str) -> &'a Item {
    module
        .all_items()
        .map(|(_, item)| item)
        .find(|item| item.name == name)
        .unwrap_or_else(|| panic!("no item `{name}`"))
}

#[test]
fn crate_is_named() {
    assert_eq!(NAME, "cpl-doc");
}

#[test]
fn a_module_is_named_after_its_file() {
    let module = lib();
    assert_eq!(
        (module.name.as_str(), module.path.as_str()),
        ("prefix", "lib/prefix.cp")
    );
    assert_eq!(extract("", "a\\b\\geo.cp").name, "geo");
}

#[test]
fn a_leading_block_set_off_by_a_blank_line_documents_the_module() {
    assert_eq!(
        lib().doc,
        "Prefix sums over a list.\n\nBuild once in O(n), then answer any range sum in O(1)."
    );
    // Without the blank line it belongs to the first declaration.
    let attached = extract(
        "/// Squares a number.\nfn sq(x: Int) -> Int = x * x\n",
        "m.cp",
    );
    assert_eq!(attached.doc, "");
    assert_eq!(attached.items[0].doc, "Squares a number.");
}

#[test]
fn every_declaration_is_listed_in_source_order_with_its_kind() {
    let module = lib();
    let listed: Vec<(Kind, &str)> = module
        .items
        .iter()
        .map(|item| (item.kind, item.name.as_str()))
        .collect();
    assert_eq!(
        listed,
        [
            (Kind::Struct, "Prefix"),
            (Kind::Enum, "Bound"),
            (Kind::Const, "LIMIT"),
            (Kind::Domain, "Small"),
            (Kind::Type, "Pair"),
            (Kind::Fn, "larger"),
        ]
    );
}

#[test]
fn signatures_are_the_declaration_without_its_body() {
    let module = lib();
    let signature = |name: &str| find(&module, name).signature.clone();
    assert_eq!(signature("Prefix"), "struct Prefix");
    assert_eq!(signature("pre"), "pre: List<Int>");
    assert_eq!(signature("of"), "fn of(a: List<Int>) -> Prefix");
    assert_eq!(signature("sum"), "fn sum(self, l: Int, r: Int) -> Int");
    assert_eq!(signature("Bound"), "enum Bound");
    assert_eq!(signature("Closed"), "Closed");
    assert_eq!(signature("Open"), "Open(Int)");
    assert_eq!(signature("LIMIT"), "const LIMIT = 1e18");
    assert_eq!(signature("Small"), "domain Small = 0..10");
    assert_eq!(signature("Pair"), "type Pair = (Int, Int)");
    // One spread over several lines reads as one.
    assert_eq!(signature("larger"), "fn larger<T>( a: T, b: T, ) -> T");
}

#[test]
fn doc_comments_attach_to_the_declaration_below_them() {
    let module = lib();
    assert_eq!(
        find(&module, "pre").doc,
        "`pre[i]` is the sum of the first `i` values."
    );
    assert_eq!(find(&module, "Closed").doc, "Includes its end.");
    assert_eq!(find(&module, "Open").doc, "");
    assert_eq!(find(&module, "Small").doc, "");
    // Above the decorators, not between them and the fn.
    assert_eq!(find(&module, "of").doc, "Builds the sums of `a`.");
}

#[test]
fn only_a_comment_with_its_line_to_itself_and_three_slashes_is_a_doc() {
    let src = "// A plain comment.\nfn a() = 1\n//// A ruler.\nfn b() = 2\nlet x = 1 /// trailing\nfn c() = 3\n/// Real.\n\nfn d() = 4\n";
    let module = extract(src, "m.cp");
    let docs: Vec<&str> = module.items.iter().map(|item| item.doc.as_str()).collect();
    // The blank line detaches `Real.` from `d`.
    assert_eq!(docs, ["", "", "", ""]);
}

#[test]
fn cost_is_read_from_the_decorator_with_or_without_quotes() {
    let module = lib();
    assert_eq!(find(&module, "of").cost.as_deref(), Some("O(n)"));
    assert_eq!(find(&module, "sum").cost.as_deref(), Some("O(1)"));
    assert_eq!(find(&module, "larger").cost, None);
}

#[test]
fn lines_point_at_the_declaration_not_its_comment() {
    let module = lib();
    assert_eq!(find(&module, "Prefix").line, 11);
    assert_eq!(find(&module, "pre").line, 13);
    // Under `@cost`.
    assert_eq!(find(&module, "of").line, 17);
    assert_eq!(
        LIB.lines().nth(16).unwrap().trim(),
        "fn of(a: List<Int>) -> Prefix:"
    );
}

#[test]
fn the_summary_is_the_first_sentence() {
    let module = lib();
    assert_eq!(find(&module, "sum").summary(), "The sum of `a[l..r]`.");
    assert_eq!(
        find(&module, "Prefix").summary(),
        "Sums of every prefix of a list."
    );
    assert_eq!(find(&module, "Open").summary(), "");
    let odd = extract("/// Rounds 1.5 up\n/// and more\nfn r() = 2\n", "m.cp");
    assert_eq!(odd.items[0].summary(), "Rounds 1.5 up\nand more");
}

#[test]
fn helpers_and_undocumented_values_are_left_out() {
    let module = extract(
        "fn _helper() = 1\nlet n: 1..=9 = read()\n/// How many tests.\nlet t: 1..=9 = read()\nprint(n + t)\n",
        "m.cp",
    );
    let names: Vec<&str> = module.items.iter().map(|item| item.name.as_str()).collect();
    assert_eq!(names, ["t"]);
    assert_eq!(module.items[0].kind, Kind::Value);
    assert_eq!(module.items[0].signature, "let t: 1..=9");
}

#[test]
fn traits_and_impls_list_their_fns() {
    let src = "/// Things with a size.\ntrait Sized:\n    /// How many.\n    fn size(self) -> Int\n\nimpl Sized for Bag:\n    fn size(self) -> Int = self.n\n";
    let module = extract(src, "m.cp");
    assert_eq!(module.items[0].signature, "trait Sized");
    assert_eq!(module.items[0].members[0].doc, "How many.");
    assert_eq!(module.items[1].kind, Kind::Impl);
    assert_eq!(module.items[1].signature, "impl Sized for Bag");
    assert_eq!(module.items[1].members[0].signature, "fn size(self) -> Int");
    assert_eq!(module.items[1].members[0].anchor(Some("Bag")), "Bag.size");
}

#[test]
fn examples_carry_their_line_and_only_cp_fences_count() {
    let module = lib();
    let example = &find(&module, "Prefix").examples[0];
    assert_eq!(
        example.code,
        "let p = Prefix.of([3, 1, 4])\nprint(p.sum(0, 2))\n"
    );
    assert_eq!(example.line, 8);
    assert_eq!(
        LIB.lines().nth(7).unwrap(),
        "/// let p = Prefix.of([3, 1, 4])"
    );
    let mixed = extract(
        "/// ```text\n/// not code (\n/// ```\n/// ```cp\n/// print(1)\n/// ```\nfn f() = 1\n",
        "m.cp",
    );
    assert_eq!(mixed.items[0].examples.len(), 1);
    assert_eq!(mixed.items[0].examples[0].code, "print(1)\n");
}

#[test]
fn examples_that_do_not_parse_fail_the_check() {
    let ok = examples::check(&lib());
    assert_eq!((ok.checked, ok.failures.len()), (1, 0));
    let src = "/// ```\n/// print(1)\n/// ```\n\n/// ```\n/// print(larger(1, 2)\n/// ```\nfn larger(a: Int, b: Int) -> Int = a\n";
    let report = examples::check(&extract(src, "m.cp"));
    assert_eq!(report.checked, 2);
    let [failure] = report.failures.as_slice() else {
        panic!("{report:#?}");
    };
    assert_eq!((failure.item.as_str(), failure.line), ("larger", 6));
    assert_eq!(failure.diagnostics[0].message, "this `(` is never closed");
}

#[test]
fn a_file_with_a_syntax_error_still_documents_the_rest() {
    let module = extract(
        "/// Fine.\nfn ok() = 1\n\nfn broken( = \n\n/// Also fine.\nfn after() = 2\n",
        "m.cp",
    );
    let names: Vec<&str> = module.items.iter().map(|item| item.name.as_str()).collect();
    assert!(
        names.contains(&"ok") && names.contains(&"after"),
        "{names:?}"
    );
}

#[test]
fn markdown_covers_what_doc_comments_use() {
    let html = markdown::render(
        "Uses `a[i]` and *care*, **always**.\nSee [the guide](g.html).\n\n# Notes\n- one `x`\n- two\n  more\n\n```\nlet x = 1 // <ok>\n```\n```text\na < b\n```\n",
    );
    assert_eq!(
        html,
        "<p>Uses <code>a[i]</code> and <em>care</em>, <strong>always</strong>. See <a href=\"g.html\">the guide</a>.</p>\n\
         <h4>Notes</h4>\n\
         <ul>\n<li>one <code>x</code></li>\n<li>two more</li>\n</ul>\n\
         <pre><code><span class=\"k\">let</span> x = <span class=\"n\">1</span> <span class=\"c\">// &lt;ok&gt;</span></code></pre>\n\
         <pre><code>a &lt; b</code></pre>\n"
    );
    // Markup that never closes is text.
    assert_eq!(
        markdown::render("2 * 3 and a ` and [x"),
        "<p>2 * 3 and a ` and [x</p>\n"
    );
    assert_eq!(markdown::render("<script>"), "<p>&lt;script&gt;</p>\n");
}

#[test]
fn highlighting_keeps_the_text_and_colors_the_tokens() {
    let html = highlight::highlight("fn sum(self, l: Int) -> Int = \"a<b\"");
    assert_eq!(
        html,
        "<span class=\"k\">fn</span> sum(<span class=\"k\">self</span>, l: <span class=\"t\">Int</span>) -&gt; <span class=\"t\">Int</span> = <span class=\"s\">&quot;a&lt;b&quot;</span>"
    );
    // Stripping the tags gives back the (escaped) source, whatever it is.
    for code in [
        "for v in V:\n    a[v] += 1 // bump\n",
        "x = = )",
        "let s = \"unclosed",
        "",
    ] {
        let mut text = highlight::highlight(code);
        while let Some(open) = text.find('<') {
            let close = text[open..].find('>').unwrap() + open;
            text.replace_range(open..=close, "");
        }
        assert_eq!(text, highlight::escape(code), "{code:?}");
    }
}

#[test]
fn the_site_has_an_index_a_stylesheet_and_a_page_per_module() {
    let modules = [
        lib(),
        extract("fn f() = 1\n", "other/prefix.cp"),
        extract("", "index.cp"),
    ];
    let options = html::Options {
        title: "Sample <lib>".to_string(),
        source_url: Some("https://example.com/blob/main/".to_string()),
    };
    let site = html::render(&modules, &options);
    let names: Vec<&str> = site.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "index.html",
            "style.css",
            "prefix.html",
            "prefix-2.html",
            "index-module.html"
        ]
    );
    let index = &site[0].1;
    assert!(
        index.contains("<title>Sample &lt;lib&gt;</title>"),
        "{index}"
    );
    assert!(index.contains(
        "<a href=\"prefix.html#Prefix\">Prefix</a></td><td>Sums of every prefix of a list.</td>"
    ));
    assert!(index.contains("<a href=\"prefix-2.html#f\">f</a>"));

    let page = &site[2].1;
    assert!(page.contains("<section class=\"item\" id=\"Prefix\" data-name=\"prefix\">"));
    assert!(
        page.contains("<section class=\"item member\" id=\"Prefix.sum\" data-name=\"prefix.sum\">")
    );
    assert!(page.contains("<span class=\"cost\" title=\"Time complexity\">O(1)</span>"));
    assert!(page.contains("href=\"https://example.com/blob/main/lib/prefix.cp#L22\">source</a>"));
    assert!(page.contains("<a class=\"here\" href=\"prefix.html\">prefix</a>"));
    assert!(page.contains("<h2 class=\"group\">Structs</h2>"));
    assert!(site[4].1.contains("This file declares nothing."));
    // Pages load nothing from the network.
    for (name, contents) in &site {
        assert!(
            !contents.contains("src=\"http") && !contents.contains("@import"),
            "{name}"
        );
    }
}

#[test]
fn without_a_source_url_there_are_no_source_links() {
    let site = html::render(&[lib()], &html::Options::default());
    assert!(!site[2].1.contains(">source</a>"));
    assert!(site[0].1.contains("<h1>Documentation</h1>"));
}

#[test]
fn json_carries_every_field_and_escapes_strings() {
    let text = json::render(&[extract(
        "/// Says \"hi\"\\n.\n@cost(\"O(1)\")\nfn hi(): print(\"hi\")\n",
        "m.cp",
    )]);
    assert_eq!(
        text,
        "{\"modules\":[{\"name\":\"m\",\"path\":\"m.cp\",\"doc\":\"\",\"examples\":[],\"items\":[\
         {\"kind\":\"fn\",\"name\":\"hi\",\"signature\":\"fn hi()\",\"doc\":\"Says \\\"hi\\\"\\\\n.\",\
         \"summary\":\"Says \\\"hi\\\"\\\\n.\",\"cost\":\"O(1)\",\"line\":3,\"examples\":[],\"members\":[]}]}]}\n"
    );
    assert_eq!(json::string("a\tb\u{1}"), "\"a\\tb\\u0001\"");
    assert_eq!(json::render(&[]), "{\"modules\":[]}\n");
}

/// Every program under `corpus/problems/` and `crates/cpl-cli/tests/cases/`
/// extracts without panicking, and every item points at a line of its file
/// that names it.
#[test]
fn extracts_the_whole_corpus() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut files, mut items) = (0, 0);
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(root.join("corpus/problems")).unwrap().flatten() {
        let spec = entry.path().join("spec.cp");
        if spec.exists() {
            paths.push(spec);
        }
    }
    for dir in [
        "crates/cpl-cli/tests/cases/ok",
        "crates/cpl-cli/tests/cases/lend",
        "crates/cpl-cli/tests/cases/fn",
        "crates/cpl-cli/tests/cases/bad",
    ] {
        if let Ok(entries) = std::fs::read_dir(root.join(dir)) {
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|e| e == "cp") {
                    paths.push(entry.path());
                }
            }
        }
    }
    for path in paths {
        let src = std::fs::read_to_string(&path).unwrap();
        let module = extract(&src, &path.display().to_string());
        files += 1;
        for (_, item) in module.all_items() {
            items += 1;
            let line = src.lines().nth(item.line as usize - 1).unwrap_or("");
            if !matches!(item.kind, Kind::Impl | Kind::Value) {
                assert!(
                    line.contains(&item.name),
                    "{}:{}: {item:?}",
                    path.display(),
                    item.line
                );
            }
            assert!(!item.signature.contains('\n'));
        }
        let site = html::render(std::slice::from_ref(&module), &html::Options::default());
        assert_eq!(site.len(), 3);
    }
    assert!(files >= 40 && items >= 40, "{files} files, {items} items");
}

mod cli {
    use crate::cli::run;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cpl-doc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::write(dir.join("lib/prefix.cp"), super::LIB).unwrap();
        dir
    }

    fn doc(args: &[&str]) -> (u8, String, String) {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn writes_the_site_and_says_where() {
        let dir = scratch("site");
        let out = dir.join("docs");
        let (code, stdout, stderr) = doc(&[
            "--out",
            out.to_str().unwrap(),
            "--title",
            "Sample",
            dir.join("lib").to_str().unwrap(),
        ]);
        assert_eq!((code, stdout.as_str()), (0, ""), "{stderr}");
        assert!(
            stderr.starts_with("documented 11 items in 1 file, 3 without a comment; open "),
            "{stderr}"
        );
        for file in ["index.html", "style.css", "prefix.html"] {
            assert!(out.join(file).is_file(), "{file}");
        }
        assert!(std::fs::read_to_string(out.join("index.html"))
            .unwrap()
            .contains("<h1>Sample</h1>"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn json_goes_to_stdout_and_writes_nothing() {
        let dir = scratch("json");
        let (code, stdout, _) = doc(&[
            "--format",
            "json",
            dir.join("lib/prefix.cp").to_str().unwrap(),
        ]);
        assert_eq!(code, 0);
        assert!(
            stdout.starts_with("{\"modules\":[{\"name\":\"prefix\""),
            "{stdout}"
        );
        assert!(!dir.join("target").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_checks_examples_and_fails_on_a_broken_one() {
        let dir = scratch("examples");
        let lib = dir.join("lib");
        let (code, stdout, stderr) = doc(&["--test", lib.to_str().unwrap()]);
        assert_eq!((code, stdout.as_str()), (0, ""));
        assert_eq!(stderr, "1 example checked, 0 failed\n");

        std::fs::write(
            lib.join("bad.cp"),
            "/// ```\n/// print(1\n/// ```\nfn f() = 1\n",
        )
        .unwrap();
        let (code, stdout, stderr) = doc(&["--test", lib.to_str().unwrap()]);
        assert_eq!(code, 1);
        assert!(
            stdout.contains("bad.cp:2: the example for `f` does not parse"),
            "{stdout}"
        );
        assert!(stdout.contains("this `(` is never closed"), "{stdout}");
        assert_eq!(stderr, "2 examples checked, 1 failed\n");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn usage_errors_exit_2() {
        let (code, stdout, _) = doc(&["--help"]);
        assert_eq!(code, 0);
        assert!(stdout.starts_with("usage: cpl doc"));
        for (args, message) in [
            (
                &["--format", "pdf"][..],
                "`--format` is html or json, not `pdf`",
            ),
            (&["--out"][..], "`--out` needs a value"),
            (&["--loud"][..], "unknown option `--loud`"),
        ] {
            let (code, _, stderr) = doc(args);
            assert_eq!(code, 2);
            assert!(stderr.starts_with(&format!("error: {message}")), "{stderr}");
        }
        let (code, _, stderr) = doc(&["/no/such/dir"]);
        assert_eq!(code, 1);
        assert!(stderr.contains("can't read /no/such/dir"), "{stderr}");
    }
}
