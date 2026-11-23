//! Lightweight zero-dependency CLI flag parser shared by `cpl-stress` and `cpl-contest`.

use std::collections::HashMap;

/// Parsed arguments: positional words, valued options (`--seed 3`), and switches (`--once`).
#[derive(Debug, Default)]
pub struct Args {
    pub positional: Vec<String>,
    values: HashMap<&'static str, String>,
    switches: Vec<&'static str>,
}

impl Args {
    /// Splits `args` into positionals, `valued` options (`--seed 3`), and
    /// `switches` (`--once`).
    pub fn parse(
        command: &str,
        args: &[String],
        valued: &[&'static str],
        switches: &[&'static str],
    ) -> Result<Args, String> {
        let mut parsed = Args::default();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if let Some(flag) = valued.iter().find(|flag| *flag == arg) {
                let value = iter
                    .next()
                    .ok_or_else(|| cpl_diag::usage::needs_value(arg))?;
                parsed.values.insert(flag, value.clone());
            } else if let Some(flag) = switches.iter().find(|flag| *flag == arg) {
                parsed.switches.push(flag);
            } else if arg.starts_with("--")
                || (arg.starts_with('-') && arg.len() > 1 && !arg.as_bytes()[1].is_ascii_digit())
            {
                return Err(cpl_diag::usage::no_option(command, arg));
            } else {
                parsed.positional.push(arg.clone());
            }
        }
        Ok(parsed)
    }

    pub fn value(&self, flag: &str) -> Option<&str> {
        self.values.get(flag).map(String::as_str)
    }

    pub fn has(&self, flag: &str) -> bool {
        self.switches.contains(&flag)
    }

    /// A whole-number option.
    pub fn number(&self, flag: &str) -> Result<Option<u64>, String> {
        self.value(flag)
            .map(|text| {
                text.parse()
                    .map_err(|_| format!("`{flag}` takes a whole number, not `{text}`"))
            })
            .transpose()
    }

    /// The one positional file/problem a command takes.
    pub fn file(&self, command: &str) -> Result<&str, String> {
        match self.positional.as_slice() {
            [file] => Ok(file),
            [] => Err(format!("`{command}` needs a problem: `cpl {command} d`")),
            [_, extra, ..] => Err(format!(
                "`{command}` takes one problem; `{extra}` is a second"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Args;

    fn words(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parses_positionals_values_and_switches() {
        let args = Args::parse(
            "stress",
            &words(&["d.cp", "--seed", "42", "--trace"]),
            &["--seed"],
            &["--trace", "--explain"],
        )
        .unwrap();
        assert_eq!(args.file("stress").unwrap(), "d.cp");
        assert_eq!(args.number("--seed").unwrap(), Some(42));
        assert!(args.has("--trace"));
        assert!(!args.has("--explain"));
    }

    #[test]
    fn rejects_missing_value_bad_number_and_unknown_option() {
        assert!(Args::parse("stress", &words(&["--seed"]), &["--seed"], &[]).is_err());
        assert!(Args::parse("stress", &words(&["--bogus"]), &[], &[]).is_err());
        assert!(Args::parse("stress", &words(&["-x"]), &[], &[]).is_err());
        let args = Args::parse("stress", &words(&["--seed", "abc"]), &["--seed"], &[]).unwrap();
        assert!(args.number("--seed").is_err());
    }
}
