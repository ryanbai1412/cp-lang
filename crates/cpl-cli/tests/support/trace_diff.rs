//! Why a saved trace differs from the one a run records, by kind: steps
//! lost or added on a source line, a step whose values (their `Show` text),
//! locals or frames changed, ids renumbered, the part past the 8 KB cut, or
//! standard error. A raw first-differing-line diff hides a lost step behind
//! every renumbered id after it; this names the step instead.

const CUT: &str = "... ";
const STDERR: &str = "stderr:\n";

/// One step of a saved trace: its id, and the rest of its JSON.
#[derive(Debug, Clone, Copy)]
struct Step<'a> {
    id: &'a str,
    body: &'a str,
}

impl<'a> Step<'a> {
    fn field(&self, key: &str) -> &'a str {
        let Some(at) = self.body.find(&format!("\"{key}\":")) else {
            return "";
        };
        let rest = &self.body[at + key.len() + 3..];
        match rest.strip_prefix('"') {
            Some(text) => {
                let mut end = 0;
                let bytes = text.as_bytes();
                while end < bytes.len()
                    && !(bytes[end] == b'"' && (end == 0 || bytes[end - 1] != b'\\'))
                {
                    end += 1;
                }
                &text[..end]
            }
            None => &rest[..rest.find([',', '}']).unwrap_or(rest.len())],
        }
    }

    fn line(&self) -> &'a str {
        self.field("line")
    }

    fn label(&self) -> &'a str {
        self.field("label")
    }
}

/// A saved trace cut into what comes before the steps, the steps, the
/// summary of the part past the cut, and standard error.
#[derive(Debug, Default)]
struct Saved<'a> {
    head: Vec<&'a str>,
    steps: Vec<Step<'a>>,
    cut: Option<&'a str>,
    stderr: &'a str,
}

fn split(text: &str) -> Saved<'_> {
    let (trace, stderr) = match text.find(&format!("\n{STDERR}")) {
        Some(at) => (&text[..at], &text[at + 1 + STDERR.len()..]),
        None => (text, ""),
    };
    let mut saved = Saved {
        stderr,
        ..Saved::default()
    };
    for line in trace.lines() {
        if line.starts_with(CUT) && line.contains(" more lines, ") {
            saved.cut = Some(line);
            continue;
        }
        let Some(at) = line.find("{\"id\":") else {
            saved.head.push(line);
            continue;
        };
        if at > 0 {
            saved.head.push(&line[..at]);
        }
        let end = at + object_len(&line[at..]);
        let rest = &line[at + 6..end];
        let digits = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        saved.steps.push(Step {
            id: &rest[..digits],
            body: &rest[digits..],
        });
        let after = line[end..].trim_start_matches(',');
        if !after.is_empty() {
            saved.head.push(after);
        }
    }
    saved
}

/// The length of the JSON object `text` starts with.
fn object_len(text: &str) -> usize {
    let (mut depth, mut string, mut escaped) = (0usize, false, false);
    for (at, c) in text.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if string => escaped = true,
            '"' => string = !string,
            '{' | '[' if !string => depth += 1,
            '}' | ']' if !string => {
                depth -= 1;
                if depth == 0 {
                    return at + 1;
                }
            }
            _ => {}
        }
    }
    text.len()
}

/// The pairs `(got, want)` of steps a longest common subsequence of
/// their bodies matches, in order.
fn matched(got: &[Step], want: &[Step]) -> Vec<(usize, usize)> {
    let (n, m) = (got.len(), want.len());
    let mut lcs = vec![0u32; (n + 1) * (m + 1)];
    let at = |i: usize, j: usize| i * (m + 1) + j;
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[at(i, j)] = if got[i].body == want[j].body {
                lcs[at(i + 1, j + 1)] + 1
            } else {
                lcs[at(i + 1, j)].max(lcs[at(i, j + 1)])
            };
        }
    }
    let (mut i, mut j, mut pairs) = (0, 0, Vec::new());
    while i < n && j < m {
        if got[i].body == want[j].body {
            pairs.push((i, j));
            i += 1;
            j += 1;
        } else if lcs[at(i + 1, j)] >= lcs[at(i, j + 1)] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
}

/// `got` and `want` around where they first differ.
fn around(got: &str, want: &str) -> (String, String) {
    let start = got
        .char_indices()
        .zip(want.chars())
        .find(|((_, g), w)| g != w)
        .map_or(got.len().min(want.len()), |((at, _), _)| at);
    let from = got[..start]
        .char_indices()
        .rev()
        .nth(40)
        .map_or(0, |(at, _)| at);
    let window = |text: &str| {
        let from = from.min(text.len());
        let from = (0..=from)
            .rev()
            .find(|&b| text.is_char_boundary(b))
            .unwrap_or(0);
        let shown: String = text[from..].chars().take(120).collect();
        format!("{}{shown}", if from > 0 { "..." } else { "" })
    };
    (window(got), window(want))
}

/// Which part of a step's JSON its first difference falls in.
fn part(got: &str, want: &str) -> &'static str {
    let at = got
        .bytes()
        .zip(want.bytes())
        .position(|(g, w)| g != w)
        .unwrap_or(got.len().min(want.len()));
    let before = &got[..at];
    [
        ("\"values\":", "the shown values (their Show text)"),
        ("\"locals\":", "the locals"),
        ("\"frames\":", "the call frames"),
        ("\"stage\":", "the stage"),
        ("\"label\":", "the label"),
        ("\"line\":", "the line"),
    ]
    .into_iter()
    .filter_map(|(key, name)| before.rfind(key).map(|at| (at, name)))
    .max_by_key(|&(at, _)| at)
    .map_or("the step", |(_, name)| name)
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Every difference between the recorded trace `got` and the saved
/// `want`, one per line, classified; empty if they agree.
pub fn classify(got: &str, want: &str) -> Vec<String> {
    if got == want {
        return Vec::new();
    }
    let (g, w) = (split(got), split(want));
    let mut found = Vec::new();
    if g.head != w.head {
        let (gh, wh) = (g.head.join("\n"), w.head.join("\n"));
        let (gs, ws) = around(&gh, &wh);
        found.push(format!(
            "the variables changed:\n      got  {gs}\n      want {ws}"
        ));
    }
    let pairs = matched(&g.steps, &w.steps);
    let cut = g.cut.is_some() || w.cut.is_some();
    // Past the last matched step of a cut trace, steps only moved across the cut.
    let (g_end, w_end) = match (cut, pairs.last()) {
        (true, Some(&(i, j))) => (i + 1, j + 1),
        _ => (g.steps.len(), w.steps.len()),
    };
    let mut lost: Vec<usize> = Vec::new();
    let mut added: Vec<usize> = Vec::new();
    let (mut i, mut j) = (0, 0);
    for &(pi, pj) in pairs.iter().chain([(g_end, w_end)].iter()) {
        added.extend(i..pi.min(g_end));
        lost.extend(j..pj.min(w_end));
        i = pi + 1;
        j = pj + 1;
    }
    // A step lost and one added on the same line with the same label is one changed step.
    let mut changed = Vec::new();
    lost.retain(|&wj| {
        let want_step = w.steps[wj];
        match added.iter().position(|&gi| {
            g.steps[gi].line() == want_step.line() && g.steps[gi].label() == want_step.label()
        }) {
            Some(k) => {
                changed.push((added.remove(k), wj));
                false
            }
            None => true,
        }
    });
    let mut lines: Vec<(&str, &str)> = lost
        .iter()
        .map(|&j| (w.steps[j].line(), w.steps[j].label()))
        .chain(
            added
                .iter()
                .map(|&i| (g.steps[i].line(), g.steps[i].label())),
        )
        .collect();
    lines.sort_by_key(|&(line, _)| line.parse::<u64>().unwrap_or(u64::MAX));
    lines.dedup();
    for (line, label) in lines {
        let lost = lost
            .iter()
            .filter(|&&j| w.steps[j].line() == line && w.steps[j].label() == label)
            .count();
        let added = added
            .iter()
            .filter(|&&i| g.steps[i].line() == line && g.steps[i].label() == label)
            .count();
        let mut what = Vec::new();
        if lost > 0 {
            what.push(format!("{lost} step{} lost", plural(lost)));
        }
        if added > 0 {
            what.push(format!("{added} step{} added", plural(added)));
        }
        found.push(format!("line {line} `{label}`: {}", what.join(", ")));
    }
    for &(gi, wj) in &changed {
        let (gs, ws) = (g.steps[gi], w.steps[wj]);
        let (got_at, want_at) = around(gs.body, ws.body);
        found.push(format!(
            "line {} `{}`, step {}: {} changed:\n      got  {got_at}\n      want {want_at}",
            gs.line(),
            gs.label(),
            ws.id,
            part(gs.body, ws.body),
        ));
    }
    if found.is_empty() {
        if let Some(&(gi, wj)) = pairs
            .iter()
            .find(|&&(gi, wj)| g.steps[gi].id != w.steps[wj].id)
        {
            found.push(format!(
                "step ids renumbered from line {} `{}`: got id {}, want id {}",
                g.steps[gi].line(),
                g.steps[gi].label(),
                g.steps[gi].id,
                w.steps[wj].id
            ));
        }
    }
    if g.cut != w.cut && found.is_empty() {
        found.push(format!(
            "past the 8 KB cut (the steps before it agree; diff the full `cpl run --trace` output):\n      got  {}\n      want {}",
            g.cut.unwrap_or("no cut"),
            w.cut.unwrap_or("no cut")
        ));
    }
    if g.stderr != w.stderr {
        let at = g
            .stderr
            .lines()
            .zip(w.stderr.lines())
            .position(|(a, b)| a != b);
        let at = at.unwrap_or(g.stderr.lines().count().min(w.stderr.lines().count()));
        let line = |text: &str| {
            text.lines()
                .nth(at)
                .unwrap_or("<end>")
                .chars()
                .take(200)
                .collect::<String>()
        };
        found.push(format!(
            "standard error, line {}:\n      got  {}\n      want {}",
            at + 1,
            line(g.stderr),
            line(w.stderr)
        ));
    }
    if found.is_empty() {
        let (gs, ws) = around(got, want);
        found.push(format!(
            "the trace changed:\n      got  {gs}\n      want {ws}"
        ));
    }
    found
}

#[cfg(test)]
mod tests {
    use super::classify;

    fn step(id: usize, line: u32, label: &str, values: &str) -> String {
        format!(
            "{{\"id\":{id},\"line\":{line},\"label\":\"{label}\",\"stage\":\"main\",\"values\":{{\"stdout\":[]{values}}},\"locals\":{{}},\"frames\":[]}}"
        )
    }

    fn trace(steps: &[String], stderr: &str) -> String {
        format!(
            "\"variables\":{{}},\"runs\":{{\"fast\":{{\"steps\":[{}]}}}}\nstderr:\n{stderr}",
            steps.join(",\n")
        )
    }

    fn renumbered(steps: &[(u32, &str, &str)]) -> Vec<String> {
        steps
            .iter()
            .enumerate()
            .map(|(id, &(line, label, values))| step(id, line, label, values))
            .collect()
    }

    #[test]
    fn a_lost_step_is_named_by_its_line_not_by_every_id_after_it() {
        let want = trace(
            &renumbered(&[(1, "a", ""), (2, "b", ""), (3, "c", ""), (3, "c", "")]),
            "",
        );
        let got = trace(&renumbered(&[(1, "a", ""), (3, "c", ""), (3, "c", "")]), "");
        assert_eq!(classify(&got, &want), ["line 2 `b`: 1 step lost"]);
    }

    #[test]
    fn added_steps_are_counted_per_line() {
        let want = trace(&renumbered(&[(1, "a", "")]), "");
        let got = trace(&renumbered(&[(1, "a", ""), (4, "d", ""), (4, "d", "")]), "");
        assert_eq!(classify(&got, &want), ["line 4 `d`: 2 steps added"]);
    }

    #[test]
    fn a_changed_show_text_names_the_step_and_the_values() {
        let want = trace(&renumbered(&[(1, "a", ",\"t\":\"Seg[1, 2]\"")]), "");
        let got = trace(&renumbered(&[(1, "a", ",\"t\":\"Seg(1, 2)\"")]), "");
        let found = classify(&got, &want);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].starts_with("line 1 `a`, step 0: the shown values (their Show text) changed:"),
            "{found:?}"
        );
        assert!(
            found[0].contains("Seg(1, 2)") && found[0].contains("Seg[1, 2]"),
            "{found:?}"
        );
    }

    #[test]
    fn renumbered_ids_alone_are_named_as_such() {
        let want = trace(&[step(0, 1, "a", ""), step(1, 2, "b", "")], "");
        let got = trace(&[step(0, 1, "a", ""), step(2, 2, "b", "")], "");
        assert_eq!(
            classify(&got, &want),
            ["step ids renumbered from line 2 `b`: got id 2, want id 1"]
        );
    }

    #[test]
    fn standard_error_is_compared_on_its_own() {
        let steps = renumbered(&[(1, "a", "")]);
        let found = classify(
            &trace(&steps, "E0101 oops\n"),
            &trace(&steps, "E0102 oops\n"),
        );
        assert_eq!(
            found,
            ["standard error, line 1:\n      got  E0101 oops\n      want E0102 oops"]
        );
    }

    #[test]
    fn steps_moved_across_the_cut_are_not_lost() {
        let cut = |n: usize| format!("... {n} more lines, 10 bytes, fnv64 00000000000000{n:02}");
        let want = trace(&renumbered(&[(1, "a", ""), (2, "b", ""), (3, "c", "")]), "");
        let want = want.replacen("\nstderr:", &format!("\n{}\nstderr:", cut(1)), 1);
        let got = trace(&renumbered(&[(1, "a", ""), (2, "b", "")]), "");
        let got = got.replacen("\nstderr:", &format!("\n{}\nstderr:", cut(2)), 1);
        let found = classify(&got, &want);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with("past the 8 KB cut"), "{found:?}");
    }

    #[test]
    fn agreeing_traces_have_no_difference() {
        let t = trace(&renumbered(&[(1, "a", "")]), "");
        assert!(classify(&t, &t).is_empty());
    }
}
