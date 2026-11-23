//! The wording of usage errors, shared by every `cpl` command so a typo'd
//! option reads the same whichever command it was given to (exit code 2).

/// `flag` is none of `command`'s options.
pub fn no_option(command: &str, flag: &str) -> String {
    format!("`{command}` has no option `{flag}`; `cpl {command} --help` lists them")
}

/// `flag` came last, without the value it takes.
pub fn needs_value(flag: &str) -> String {
    format!("`{flag}` needs a value")
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_unknown_option_names_the_command_and_its_help() {
        assert_eq!(
            super::no_option("test", "--loud"),
            "`test` has no option `--loud`; `cpl test --help` lists them"
        );
        assert_eq!(super::needs_value("--seed"), "`--seed` needs a value");
    }
}
