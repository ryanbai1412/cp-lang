//! The `sizeof` audit (plan/80-validation.md, 8.2).
//!
//! A probe file is a `.cp` program whose top-level variables have the
//! types to measure, with the size the plan promises written beside them:
//!
//! ```text
//! var parent: [V]?V = none      //! sizeof parent[] == 4
//! ```
//!
//! `name` measures the variable itself and `name[]` one element of it.
//! The audit transpiles the probe, asks the C++ compiler for the real
//! sizes and compares.

use crate::backend::{Cxx, Transpiler};
use crate::process;
use std::fmt::Write;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    /// The variable, with `[]` appended for "one element of".
    pub what: String,
    /// Bytes the plan promises.
    pub want: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measured {
    pub probe: Probe,
    pub size: u64,
}

/// Reads the `//! sizeof <what> == <bytes>` annotations of a probe file.
pub fn parse_probes(source: &str) -> Result<Vec<Probe>, String> {
    let mut probes = vec![];
    for (i, line) in source.lines().enumerate() {
        let Some((_, note)) = line.split_once("//! sizeof ") else {
            continue;
        };
        let parsed = note.split_once("==").and_then(|(what, want)| {
            let what = what.trim();
            let name = what.strip_suffix("[]").unwrap_or(what);
            let is_name = !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_');
            Some(Probe {
                what: is_name.then(|| what.to_string())?,
                want: want.trim().parse().ok()?,
            })
        });
        probes.push(
            parsed.ok_or_else(|| format!("line {}: expected `//! sizeof name == bytes`", i + 1))?,
        );
    }
    Ok(probes)
}

/// C++ appended to the emitted program: prints every size from a static
/// initializer and exits before `main` can wait for input.
fn probe_code(probes: &[Probe]) -> String {
    let mut code = String::from("\nnamespace { struct CpLayoutProbe { CpLayoutProbe() {\n");
    for p in probes {
        let ty = match p.what.strip_suffix("[]") {
            Some(name) => format!("decltype({name})::value_type"),
            None => format!("decltype({})", p.what),
        };
        writeln!(
            code,
            "    std::printf(\"{} %zu\\n\", sizeof({ty}));",
            p.what
        )
        .unwrap();
    }
    code.push_str("    std::fflush(stdout); std::_Exit(0);\n} } cp_layout_probe; }\n");
    code
}

/// Measures every probe of `source`. `work` is a scratch directory.
pub fn audit(
    transpiler: &dyn Transpiler,
    cxx: &Cxx,
    source: &Path,
    work: &Path,
) -> Result<Vec<Measured>, String> {
    let text = std::fs::read_to_string(source).map_err(|e| format!("{}: {e}", source.display()))?;
    let probes = parse_probes(&text)?;
    if probes.is_empty() {
        return Err(format!(
            "{} has no `//! sizeof` annotations",
            source.display()
        ));
    }
    let (cpp, exe) = (work.join("layout.cpp"), work.join("layout"));
    transpiler.transpile(source, &cpp)?;
    let mut emitted = std::fs::read_to_string(&cpp).map_err(|e| e.to_string())?;
    emitted.push_str(&probe_code(&probes));
    std::fs::write(&cpp, emitted).map_err(|e| e.to_string())?;
    cxx.compile(&cpp, &exe)?;
    let out = process::run(&mut Command::new(&exe), b"", Duration::from_secs(20))
        .map_err(|e| e.to_string())?;
    if !out.ok() {
        return Err(format!(
            "the probe exited with {:?}: {}",
            out.exit, out.stderr
        ));
    }
    let sizes: Vec<u64> = out
        .stdout
        .lines()
        .filter_map(|l| l.rsplit_once(' ')?.1.parse().ok())
        .collect();
    if sizes.len() != probes.len() {
        return Err(format!(
            "expected {} sizes, the probe printed:\n{}",
            probes.len(),
            out.stdout
        ));
    }
    Ok(probes
        .into_iter()
        .zip(sizes)
        .map(|(probe, size)| Measured { probe, size })
        .collect())
}

/// Sizes a backend is known to miss, one `<what> <bytes>` per line with
/// `#` comments. The audit fails when a size is off and not listed here,
/// when it is listed with a different number, and when it is listed but
/// now meets the plan: every change in layout is seen.
pub fn parse_gaps(text: &str) -> Result<Vec<(String, u64)>, String> {
    text.lines()
        .map(|l| l.split('#').next().unwrap().trim())
        .filter(|l| !l.is_empty())
        .map(|l| {
            let (what, size) = l
                .split_once(char::is_whitespace)
                .ok_or(format!("expected `name bytes`: `{l}`"))?;
            Ok((
                what.to_string(),
                size.trim()
                    .parse()
                    .map_err(|_| format!("bad size: `{l}`"))?,
            ))
        })
        .collect()
}

/// Everything wrong with `measured`, given the known `gaps`. Empty means
/// the audit passes.
pub fn problems(measured: &[Measured], gaps: &[(String, u64)]) -> Vec<String> {
    let mut out = vec![];
    for m in measured {
        let Probe { what, want } = &m.probe;
        let gap = gaps.iter().find(|(w, _)| w == what).map(|g| g.1);
        match (m.size == *want, gap) {
            (true, None) => {}
            (true, Some(_)) => out.push(format!(
                "{what} is now {want} bytes as planned: remove it from the gaps file"
            )),
            (false, None) => out.push(format!("{what} is {} bytes, the plan says {want}", m.size)),
            (false, Some(g)) if g == m.size => {}
            (false, Some(g)) => out.push(format!(
                "{what} is {} bytes; the plan says {want} and the gaps file says {g}",
                m.size
            )),
        }
    }
    for (what, _) in gaps {
        if !measured.iter().any(|m| m.probe.what == *what) {
            out.push(format!(
                "the gaps file lists {what}, which the probe file does not measure"
            ));
        }
    }
    out
}

pub fn report(measured: &[Measured]) -> String {
    let mut out = String::new();
    for m in measured {
        let verdict = if m.size == m.probe.want {
            "as planned".to_string()
        } else {
            format!("plan says {}", m.probe.want)
        };
        writeln!(
            out,
            "sizeof {:16} {:3} bytes  ({verdict})",
            m.probe.what, m.size
        )
        .unwrap();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measured(what: &str, want: u64, size: u64) -> Measured {
        Measured {
            probe: Probe {
                what: what.into(),
                want,
            },
            size,
        }
    }

    #[test]
    fn annotations_are_read_from_the_probe_file() {
        let src = "var pos: V = V(1)   //! sizeof pos == 4\nvar es: List<E> = []  //! sizeof es[] == 12\nprint(1)\n";
        assert_eq!(
            parse_probes(src).unwrap(),
            [
                Probe {
                    what: "pos".into(),
                    want: 4
                },
                Probe {
                    what: "es[]".into(),
                    want: 12
                }
            ]
        );
        assert!(parse_probes("//! sizeof a b == 4").is_err());
        assert!(parse_probes("//! sizeof a == four").is_err());
    }

    #[test]
    fn the_probe_measures_values_and_elements() {
        let code = probe_code(&parse_probes("//! sizeof pos == 4\n//! sizeof es[] == 12").unwrap());
        assert!(code.contains("sizeof(decltype(pos))"));
        assert!(code.contains("sizeof(decltype(es)::value_type)"));
    }

    #[test]
    fn every_layout_change_is_a_problem() {
        let gaps = parse_gaps("# known\nb 8  # Opt<int> is not niche-packed\nc 8\n").unwrap();
        // As planned; a known gap; an unlisted miss; a gap that moved; a gap that closed.
        assert!(problems(
            &[
                measured("a", 4, 4),
                measured("b", 4, 8),
                measured("c", 4, 8)
            ],
            &gaps
        )
        .is_empty());
        assert_eq!(problems(&[measured("a", 4, 8)], &[]).len(), 1);
        assert_eq!(
            problems(&[measured("b", 4, 16), measured("c", 4, 8)], &gaps).len(),
            1
        );
        assert_eq!(
            problems(&[measured("b", 4, 4), measured("c", 4, 8)], &gaps).len(),
            1
        );
        assert_eq!(
            problems(&[measured("b", 4, 8)], &gaps).len(),
            1,
            "c is listed but not measured"
        );
    }
}
