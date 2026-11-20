//! Command-line arguments shared by the contest commands.

use std::collections::HashMap;

/// Parsed arguments: the words that are not options, and the options.
#[derive(Debug, Default)]
pub(crate) struct Args {
    pub positional: Vec<String>,
    values: HashMap<&'static str, String>,
    switches: Vec<&'static str>,
}

impl Args {
    /// Splits `args` into positionals, `valued` options (`--seed 3`) and
    /// `switches` (`--once`).
    pub fn parse(
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
                    .ok_or_else(|| format!("`{arg}` needs a value"))?;
                parsed.values.insert(flag, value.clone());
            } else if let Some(flag) = switches.iter().find(|flag| *flag == arg) {
                parsed.switches.push(flag);
            } else if arg.starts_with("--") {
                return Err(format!("unknown option `{arg}`"));
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

    /// The one file a command takes.
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
