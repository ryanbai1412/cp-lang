//! Line counts, measured two ways (plan/80-validation.md, 8.1).
//!
//! - Raw: lines of the port against lines of the accepted C++ submission.
//! - Library-stripped: lines of the port against the submission with
//!   everything a perfect C++ contest library would have supplied taken
//!   out. Those regions are written down by hand in a `.strip` file next
//!   to the port, so the baseline can be reviewed and argued with.
//!
//! Blank lines and comments never count, on either side.

use std::fmt::Write;
use std::path::Path;

/// The 1-based numbers of the lines that hold code: not blank, and not
/// only a comment. `//` and `/* */` comments are understood, and so are
/// string and character literals that contain comment markers.
pub fn code_lines(text: &str) -> Vec<usize> {
    #[derive(PartialEq)]
    enum State {
        Code,
        Block,
        Str(char),
    }
    let mut state = State::Code;
    let mut lines = vec![];
    for (i, line) in text.lines().enumerate() {
        let mut has_code = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            match state {
                State::Block => {
                    if c == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        state = State::Code;
                    }
                }
                State::Str(quote) => {
                    has_code = true;
                    if c == '\\' {
                        chars.next();
                    } else if c == quote {
                        state = State::Code;
                    }
                }
                State::Code => match c {
                    '/' if chars.peek() == Some(&'/') => break,
                    '/' if chars.peek() == Some(&'*') => {
                        chars.next();
                        state = State::Block;
                    }
                    '"' | '\'' => {
                        has_code = true;
                        state = State::Str(c);
                    }
                    c if c.is_whitespace() => {}
                    _ => has_code = true,
                },
            }
        }
        // A string or character literal never continues on the next line.
        if matches!(state, State::Str(_)) {
            state = State::Code;
        }
        if has_code {
            lines.push(i + 1);
        }
    }
    lines
}

/// Lines `first..=last` of the original stand for `facility`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub first: usize,
    pub last: usize,
    pub facility: String,
}

/// Parses a `.strip` file: one `first-last  facility` per line, with `#`
/// comments. Regions must be in order and must not overlap.
pub fn parse_strip(text: &str) -> Result<Vec<Region>, String> {
    let mut regions: Vec<Region> = vec![];
    for (i, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let err = |what: &str| format!("line {}: {what}: `{line}`", i + 1);
        let (range, facility) = line
            .split_once(char::is_whitespace)
            .ok_or_else(|| err("expected `first-last facility`"))?;
        let (first, last) = range.split_once('-').unwrap_or((range, range));
        let first: usize = first.parse().map_err(|_| err("bad line number"))?;
        let last: usize = last.parse().map_err(|_| err("bad line number"))?;
        if first == 0 || first > last {
            return Err(err("empty or backwards range"));
        }
        if regions.last().is_some_and(|prev| prev.last >= first) {
            return Err(err("overlaps or precedes the region before it"));
        }
        regions.push(Region {
            first,
            last,
            facility: facility.trim().to_string(),
        });
    }
    Ok(regions)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortLoc {
    pub name: String,
    /// Code lines of the `.cp` port.
    pub port: usize,
    /// Code lines of the accepted C++ submission.
    pub original: usize,
    /// Code lines of the submission outside the stripped regions, and the
    /// code lines each facility accounted for. `None` when nobody has
    /// written the `.strip` file yet.
    pub stripped: Option<(usize, Vec<(String, usize)>)>,
}

/// Percent fewer lines than `baseline`; negative when the port is longer.
pub fn reduction(port: usize, baseline: usize) -> f64 {
    if baseline == 0 {
        return 0.0;
    }
    100.0 * (1.0 - port as f64 / baseline as f64)
}

pub fn measure(
    name: &str,
    port: &str,
    original: &str,
    strip: Option<&str>,
) -> Result<PortLoc, String> {
    let original_lines = code_lines(original);
    let stripped = match strip {
        None => None,
        Some(text) => {
            let regions = parse_strip(text)?;
            let total = original.lines().count();
            if let Some(r) = regions.iter().find(|r| r.last > total) {
                return Err(format!(
                    "region {}-{} is past the end ({total} lines)",
                    r.first, r.last
                ));
            }
            let inside = |r: &Region| {
                original_lines
                    .iter()
                    .filter(|&&l| r.first <= l && l <= r.last)
                    .count()
            };
            let facilities: Vec<(String, usize)> = regions
                .iter()
                .map(|r| (r.facility.clone(), inside(r)))
                .collect();
            let removed: usize = facilities.iter().map(|(_, n)| n).sum();
            Some((original_lines.len() - removed, facilities))
        }
    };
    Ok(PortLoc {
        name: name.to_string(),
        port: code_lines(port).len(),
        original: original_lines.len(),
        stripped,
    })
}

/// Measures every `<name>.cp` in `dir` that has a `<name>_orig.cpp`,
/// reading `<name>.strip` when it exists.
pub fn measure_dir(dir: &Path) -> Result<Vec<PortLoc>, String> {
    let read = |p: &Path| std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()));
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter_map(|file| file.strip_suffix(".cp").map(str::to_string))
        .filter(|name| dir.join(format!("{name}_orig.cpp")).is_file())
        .collect();
    names.sort();
    names
        .iter()
        .map(|name| {
            let strip_path = dir.join(format!("{name}.strip"));
            let strip = strip_path
                .is_file()
                .then(|| read(&strip_path))
                .transpose()?;
            measure(
                name,
                &read(&dir.join(format!("{name}.cp")))?,
                &read(&dir.join(format!("{name}_orig.cpp")))?,
                strip.as_deref(),
            )
            .map_err(|e| format!("{}: {e}", strip_path.display()))
        })
        .collect()
}

/// The report: one line per port, then the totals.
pub fn report(ports: &[PortLoc]) -> String {
    let mut out = String::new();
    let width = ports.iter().map(|p| p.name.len()).max().unwrap_or(0);
    let line =
        |out: &mut String, name: &str, port: usize, original: usize, stripped: Option<usize>| {
            write!(
                out,
                "{name:width$}  {port:4} lines, original {original:4} ({:+.0}%)",
                -reduction(port, original)
            )
            .unwrap();
            match stripped {
                Some(s) => writeln!(
                    out,
                    ", library-stripped {s:4} ({:+.0}%)",
                    -reduction(port, s)
                )
                .unwrap(),
                None => writeln!(out, ", library-stripped: no .strip file yet").unwrap(),
            }
        };
    for p in ports {
        line(
            &mut out,
            &p.name,
            p.port,
            p.original,
            p.stripped.as_ref().map(|s| s.0),
        );
    }
    let reviewed: Vec<&PortLoc> = ports.iter().filter(|p| p.stripped.is_some()).collect();
    let sum =
        |f: &dyn Fn(&PortLoc) -> usize, ps: &[&PortLoc]| ps.iter().map(|p| f(p)).sum::<usize>();
    let all: Vec<&PortLoc> = ports.iter().collect();
    writeln!(
        out,
        "\nraw: {} ports, {:.1}% fewer lines than the originals",
        ports.len(),
        reduction(sum(&|p| p.port, &all), sum(&|p| p.original, &all))
    )
    .unwrap();
    if reviewed.is_empty() {
        writeln!(out, "library-stripped: no port has a .strip file yet").unwrap();
    } else {
        writeln!(
            out,
            "library-stripped: {} of {} ports, {:.1}% fewer lines than the originals without library code",
            reviewed.len(),
            ports.len(),
            reduction(sum(&|p| p.port, &reviewed), sum(&|p| p.stripped.as_ref().unwrap().0, &reviewed))
        )
        .unwrap();
        let mut by_facility: Vec<(String, usize)> = vec![];
        for (facility, n) in reviewed
            .iter()
            .flat_map(|p| &p.stripped.as_ref().unwrap().1)
        {
            match by_facility.iter_mut().find(|(f, _)| f == facility) {
                Some(entry) => entry.1 += n,
                None => by_facility.push((facility.clone(), *n)),
            }
        }
        by_facility.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        for (facility, n) in by_facility {
            writeln!(out, "  stripped {n:4} lines of {facility}").unwrap();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_lines_and_comments_do_not_count() {
        let cpp = "\
// header comment
#include <bits/stdc++.h>

/* a block
   comment */ int x; // trailing
int main() { /* inline */ }
  /* only a comment */
";
        assert_eq!(code_lines(cpp), [2, 5, 6]);
    }

    #[test]
    fn comment_markers_inside_literals_are_code() {
        assert_eq!(
            code_lines("puts(\"// not a comment\");\nchar c = '/';\nputs(\"/*\");\nint y;"),
            [1, 2, 3, 4]
        );
        assert_eq!(code_lines("print(\"a\\\"// b\")"), [1]);
    }

    #[test]
    fn strip_file_is_parsed_and_validated() {
        let regions = parse_strip(
            "# header\n1-12  template header\n\n20-45 segment tree # hand-written\n50 fast I/O\n",
        )
        .unwrap();
        assert_eq!(regions.len(), 3);
        assert_eq!(
            regions[1],
            Region {
                first: 20,
                last: 45,
                facility: "segment tree".into()
            }
        );
        assert_eq!((regions[2].first, regions[2].last), (50, 50));
        assert!(parse_strip("5-3 x").is_err());
        assert!(parse_strip("1-5 a\n5-9 b").is_err());
        assert!(parse_strip("1-5").is_err());
    }

    #[test]
    fn both_reductions_are_reported() {
        let original = "#define ll long long\n#define pb push_back\n\nint main() {\n  // solve\n  return 0;\n}\n";
        let port = "// port\nprint(0)\n";
        let loc = measure("p", port, original, Some("1-2 template header\n")).unwrap();
        assert_eq!((loc.port, loc.original), (1, 5));
        assert_eq!(loc.stripped, Some((3, vec![("template header".into(), 2)])));
        assert_eq!(reduction(loc.port, loc.original), 80.0);
        let text = report(&[loc]);
        assert!(text.contains("original    5 (-80%)"), "{text}");
        assert!(text.contains("library-stripped    3 (-67%)"), "{text}");
        assert!(measure("p", port, original, Some("1-99 x")).is_err());
    }

    #[test]
    fn a_port_without_a_strip_file_is_not_counted_as_stripped() {
        let loc = measure("p", "print(0)\n", "int main() {}\n", None).unwrap();
        assert_eq!(loc.stripped, None);
        assert!(report(&[loc]).contains("no .strip file yet"));
    }
}
