//! The small programs, inputs and pages the command-output tests run every
//! `cpl` command on, written to a fresh folder under the target dir.

use std::path::{Path, PathBuf};

pub fn fixtures(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    std::fs::create_dir_all(dir.join("bad_index.tests")).unwrap();
    std::fs::create_dir_all(dir.join("home")).unwrap();
    let files: &[(&str, &str)] = &[
        ("a.cp", "let n: Int = read()\nprint(n * 2)\n"),
        ("v.cp", "use std.prefix.{Prefix}\nprint(1)\n"),
        ("a.tests/1.in", "3\n"),
        ("a.tests/1.out", "6\n"),
        ("bad.cp", "let x: Int = \"s\"\n"),
        ("j.cp", "let n: Int = read()\nprint(n)\n\n@check\nfn check():\n    let n: Int = read()\n    let got: Int = read_output()\n    assert got == n\n"),
        ("j.in", "4\n"),
        ("j.right", "4\n"),
        ("j.wrong", "5\n"),
        ("same.cp", "let n: 1..=10 = read()\nprint(n * 2)\n\n@alt(program)\nfn slow():\n    let m: 1..=10 = read()\n    print(m + m)\n"),
        ("differ.cp", "let n: 1..=10 = read()\nif n > 5:\n    print(n)\nelse:\n    print(n * 2)\n\n@alt(program)\nfn slow():\n    let m: 1..=10 = read()\n    print(m + m)\n"),
        ("alt.cp", "fn total(n: Int) -> Int = n + 5\n@alt(total)\nfn total_slow(n: Int) -> Int = 5 + n\nlet n: 1..=8 = read()\nprint(total(n))\n"),
        ("bad_index.cp", "let xs = [1, 2]\nlet n: Int = read()\nprint(xs[n])\n"),
        ("bad_index.tests/1.in", "5\n"),
        ("r.cp", "fn sq(x: Int) -> Int:\n    return x * x\nprint(sq(3))\n"),
        ("page.html", "<span class=\"h2\">\nA - Sum\n</span>\n<p>Time Limit: 2 sec / Memory Limit: 1024 MiB</p>\n<span class=\"lang-en\">\n<h3>Sample Input 1</h3><pre>1 2\n</pre><h3>Sample Output 1</h3><pre>3\n</pre></span>"),
    ];
    for (name, text) in files {
        std::fs::write(dir.join(name), text).unwrap();
    }
    dir
}
