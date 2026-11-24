//! `cpl vendor`: the std modules a file loads, copied beside it (Rbundle.6).

use std::io::Write;
use std::path::Path;

pub(crate) const USAGE: &str = "\
usage: cpl vendor <file.cp>

Copies each std module the file loads, as this compiler carries it, to
vendor/std/ beside the file, and lists them.
";

/// The std modules a check of `source` loads, by name (`\"heap\"`), and
/// those their bodies load in turn.
pub(crate) fn modules(name: &str, source: &str) -> Vec<String> {
    let mut have: Vec<String> = Vec::new();
    loop {
        let given = have.clone();
        let more: Vec<String> = cpl_check::std_needed_with(name, source, move |m| {
            given
                .iter()
                .any(|g| g == m)
                .then(|| cpl_check::bundled_std(&[m.to_string()]))
                .flatten()
        })
        .into_iter()
        .filter(|m| !have.contains(m))
        .collect();
        if more.is_empty() {
            have.sort();
            return have;
        }
        have.extend(more);
    }
}

pub(crate) fn run(file: &str, stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let source = match std::fs::read_to_string(file) {
        Ok(source) => source,
        Err(e) => {
            *stderr += &format!("cpl: cannot read `{file}`: {e}\n");
            return crate::EXIT_ERRORS;
        }
    };
    let dir = Path::new(file)
        .parent()
        .unwrap_or(Path::new(""))
        .join("vendor")
        .join("std");
    for module in modules(file, &source) {
        let Some(text) = cpl_check::bundled_std(std::slice::from_ref(&module)) else {
            *stderr += &format!("cpl: this compiler does not carry `std.{module}`\n");
            return crate::EXIT_ERRORS;
        };
        let path = dir.join(format!("{module}.cp"));
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&path, text));
        if let Err(e) = written {
            *stderr += &format!("cpl: cannot write `{}`: {e}\n", path.display());
            return crate::EXIT_ERRORS;
        }
        let _ = writeln!(
            stdout,
            "std.{} -> {}",
            module.replace('/', "."),
            path.display()
        );
    }
    0
}

#[cfg(test)]
mod tests {
    #[test]
    fn copies_each_loaded_module_as_carried() {
        let dir = std::env::temp_dir().join(format!("cpl-vendor-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.cp");
        std::fs::write(
            &file,
            "use std.heap.{MinMaxHeap}\nvar h = MinMaxHeap<Int>()\nh.push(3)\nprint(h.len)\n",
        )
        .unwrap();
        let (mut out, mut err) = (Vec::new(), String::new());
        assert_eq!(
            super::run(file.to_str().unwrap(), &mut out, &mut err),
            0,
            "{err}"
        );
        let copied = std::fs::read_to_string(dir.join("vendor/std/heap.cp")).unwrap();
        assert_eq!(
            Some(copied.as_str()),
            cpl_check::bundled_std(&["heap".into()])
        );
        assert!(String::from_utf8(out).unwrap().contains("std.heap -> "));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
