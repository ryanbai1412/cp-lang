//! Judge profiles: what a submission meets on a judge. The compiler and
//! the flags it is built with, the platform, the stack the program gets,
//! and the source size the judge accepts.
//!
//! A file names its judge with `// judge: <name>`; a project with
//! `judge = "<name>"` under `[judge]` in the nearest `cp.toml`; a build
//! with `--judge <name>`, which wins over both. Without any, the judge is
//! `generic`. A `[judge.<name>]` table in `cp.toml` changes a profile's
//! fields, or adds a profile when it says what it `extends`.

use std::path::Path;

/// How much stack the judge gives the program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stack {
    Bytes(u64),
    Unlimited,
    /// Not documented: the 8 MB Linux default is assumed.
    Unknown,
}

impl Stack {
    /// The stack the program is sure to have, `None` when unlimited.
    pub fn assured(self) -> Option<u64> {
        match self {
            Stack::Bytes(bytes) => Some(bytes),
            Stack::Unlimited => None,
            Stack::Unknown => Some(8 << 20),
        }
    }

    pub fn describe(self) -> String {
        match self {
            Stack::Bytes(bytes) => format!("{} MB stack", bytes >> 20),
            Stack::Unlimited => "unlimited stack".into(),
            Stack::Unknown => "stack not documented (8 MB assumed)".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    Linux,
    Windows,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    /// The judge's name for its compiler, e.g. `GNU G++20 13.2 (64 bit, winlibs)`.
    pub title: String,
    /// The compiler command and the flags the judge builds with.
    pub compiler: String,
    pub flags: Vec<String>,
    pub os: Os,
    pub stack: Stack,
    /// The longest source the judge accepts, in bytes.
    pub source_limit: usize,
    /// Whether the compiler has `__int128`.
    pub int128: bool,
    /// Where these facts come from, and which are unconfirmed.
    pub evidence: String,
}

/// Stack the generated program needs before it can skip switching to a
/// stack of its own for deep recursion.
pub const DEEP_STACK: u64 = 256 << 20;

impl Profile {
    /// The source size worth a warning: three quarters of the limit.
    pub fn warn_at(&self) -> usize {
        self.source_limit / 4 * 3
    }

    /// Whether deep recursion fits the judge's own stack.
    pub fn deep_stack(&self) -> bool {
        self.stack.assured().is_none_or(|bytes| bytes >= DEEP_STACK)
    }

    /// The command line, as the judge runs it.
    pub fn command(&self) -> String {
        let mut line = self.compiler.clone();
        for flag in &self.flags {
            line.push(' ');
            line.push_str(flag);
        }
        line
    }

    pub fn stack_text(&self) -> String {
        self.stack.describe()
    }

    /// One line for the header of the C++ written for this judge.
    pub fn stamp(&self) -> String {
        format!("judge `{}`: {}; {}", self.name, self.command(), self.stack.describe())
    }

    /// What the file uses that this judge's compiler lacks.
    pub fn unsupported(&self, cpp: &str) -> Option<String> {
        if !self.int128 && cpp.contains("__int128") {
            return Some(format!(
                "`I128` is `__int128` in C++, which the compiler of judge `{}` ({}) does not have",
                self.name, self.title
            ));
        }
        None
    }
}

/// The judge's build as this machine can run it: the compiler, the
/// judge's flags it accepts, and the ones it does not.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Local {
    pub compiler: String,
    pub flags: Vec<String>,
    /// Each judge flag this compiler rejects, and what is used instead.
    pub dropped: Vec<(String, Option<String>)>,
}

impl Profile {
    /// The compiler that builds for the judge's platform here: a MinGW
    /// cross compiler for a Windows judge when there is one.
    pub fn local_compiler(&self) -> String {
        if self.os == Os::Windows && cfg!(not(windows)) {
            let cross = "x86_64-w64-mingw32-g++";
            if accepts(cross, None) {
                return cross.into();
            }
        }
        self.compiler.clone()
    }

    /// The judge's build on this machine. A flag the compiler here rejects
    /// is dropped; a `-std=` it rejects becomes the older spelling of the
    /// same standard (`gnu++2b` for `gnu++23`) when that is accepted.
    pub fn local(&self) -> Local {
        self.local_with(self.local_compiler())
    }

    /// The judge's build as a program run on this machine: the compiler
    /// here, for a Windows judge too, with the judge's flags it accepts.
    pub fn host(&self) -> Local {
        let compiler = if accepts(&self.compiler, None) { self.compiler.clone() } else { "g++".into() };
        self.local_with(compiler)
    }

    /// The `ulimit -s` that gives a program the judge's stack.
    pub fn ulimit(&self) -> String {
        match self.stack {
            Stack::Unlimited => "unlimited".into(),
            stack => (stack.assured().unwrap_or(8 << 20) >> 10).to_string(),
        }
    }

    fn local_with(&self, compiler: String) -> Local {
        let mut local = Local { compiler: compiler.clone(), flags: Vec::new(), dropped: Vec::new() };
        if accepts_all(&compiler, &self.flags) {
            local.flags = self.flags.clone();
            return local;
        }
        for flag in groups(&self.flags) {
            if accepts_all(&compiler, &flag) {
                local.flags.extend(flag);
                continue;
            }
            let older = older_spelling(&flag[0]).filter(|older| accepts_all(&compiler, std::slice::from_ref(older)));
            local.flags.extend(older.clone());
            local.dropped.push((flag.join(" "), older));
        }
        local
    }
}

/// The flags in the units they mean something in: `-x c++` is one, and
/// so is everything from `-Wl,--whole-archive` to `-Wl,--no-whole-archive`.
fn groups(flags: &[String]) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut open = false;
    let mut takes_value = false;
    for flag in flags {
        match groups.last_mut() {
            Some(last) if open || takes_value => last.push(flag.clone()),
            _ => groups.push(vec![flag.clone()]),
        }
        takes_value = !takes_value && matches!(flag.as_str(), "-x" | "-include" | "-I" | "-L" | "-D" | "-U");
        if flag == "-Wl,--whole-archive" {
            open = true;
        } else if flag == "-Wl,--no-whole-archive" {
            open = false;
        }
    }
    groups
}

fn older_spelling(flag: &str) -> Option<String> {
    let std = flag.strip_prefix("-std=")?;
    let (dialect, year) = std.split_at(std.find("++")? + 2);
    let older = match year {
        "26" => "2c",
        "23" => "2b",
        "20" => "2a",
        "17" => "1z",
        _ => return None,
    };
    Some(format!("-std={dialect}{older}"))
}

fn accepts(compiler: &str, flag: Option<&str>) -> bool {
    let flags: Vec<String> = flag.into_iter().map(String::from).collect();
    accepts_all(compiler, &flags)
}

/// Whether `compiler` builds and links an empty program with `flags`.
/// Asked once per compiler and flags.
fn accepts_all(compiler: &str, flags: &[String]) -> bool {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    type Seen = Mutex<HashMap<(String, Vec<String>), bool>>;
    static SEEN: OnceLock<Seen> = OnceLock::new();
    let key = (compiler.to_string(), flags.to_vec());
    if let Some(&seen) = SEEN.get_or_init(Default::default).lock().unwrap().get(&key) {
        return seen;
    }
    let dir = std::env::temp_dir().join(format!("cpl-judge-probe-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let main = dir.join("main.cpp");
    let _ = std::fs::write(&main, "int main() { return 0; }\n");
    let mut command = std::process::Command::new(compiler);
    command.args(flags).arg(&main).arg("-o").arg(dir.join(format!("probe{}", hash(&key))));
    // -Werror: a flag the compiler only warns about does nothing.
    command.arg("-Werror").stderr(std::process::Stdio::null()).stdout(std::process::Stdio::null());
    let ok = command.status().is_ok_and(|status| status.success());
    SEEN.get_or_init(Default::default).lock().unwrap().insert(key, ok);
    ok
}

fn hash(key: &(String, Vec<String>)) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

fn profile(
    name: &str,
    title: &str,
    command: &str,
    os: Os,
    stack: Stack,
    source_limit: usize,
    evidence: &str,
) -> Profile {
    let mut words = command.split_whitespace().map(String::from);
    Profile {
        name: name.into(),
        title: title.into(),
        compiler: words.next().unwrap_or_default(),
        flags: words.collect(),
        os,
        stack,
        source_limit,
        int128: true,
        evidence: evidence.into(),
    }
}

/// The profiles cpl knows, `generic` first.
pub fn builtins() -> Vec<Profile> {
    const KB: usize = 1024;
    const MB: u64 = 1 << 20;
    vec![
        profile(
            "generic",
            "C++17 with g++",
            "g++ -std=c++17 -O2",
            Os::Linux,
            Stack::Bytes(8 * MB),
            64 * KB,
            "the floor most judges clear: C++17, the 8 MB Linux default stack, a 64 KB source",
        ),
        profile(
            "codeforces",
            "GNU G++20 13.2 (64 bit, winlibs)",
            "g++ -std=c++20 -O2 -DONLINE_JUDGE -static -Wl,--stack=268435456",
            Os::Windows,
            Stack::Bytes(256 * MB),
            64 * KB,
            "unconfirmed: the Codeforces compiler list and its 256 MB stack link flag are behind a \
             browser check",
        ),
        profile(
            "atcoder",
            "C++23 (GCC 15.2.0)",
            "g++ -std=gnu++23 -O2 -Wall -Wextra -DONLINE_JUDGE -DATCODER -DNOMINMAX -march=native \
             -fconstexpr-depth=1024 -fconstexpr-loop-limit=524288 -fconstexpr-ops-limit=2097152 \
             -fmodules -ftrivial-auto-var-init=zero -pthread",
            Os::Linux,
            Stack::Unknown,
            512 * KB,
            "the compile command is from AtCoder's language list (the flags that change what \
             compiles; its library and link flags are left out); the 512 KiB source limit and the \
             stack are unconfirmed",
        ),
        profile(
            "cses",
            "C++17 (g++ 13.3.0)",
            "g++ -std=c++17 -O2 -Wall",
            Os::Linux,
            Stack::Unknown,
            128 * KB,
            "compiler, flags and the 128 kB source limit are from cses.fi/howto; the stack is not \
             documented",
        ),
        profile(
            "icpc",
            "DOMjudge's default C++ (g++)",
            "g++ -x c++ -Wall -O2 -static -pipe",
            Os::Linux,
            Stack::Unlimited,
            256 * KB,
            "DOMjudge's default build script, its runguard (stack unlimited) and its default \
             256 KB source limit; a contest's own image sets the g++ version and so the default \
             standard",
        ),
        profile(
            "kattis",
            "C++ (g++ 15.2.0)",
            "g++ -g -O2 -std=gnu++23 -static -lrt -Wl,-u,_IO_wfile_doallocate -Wl,--whole-archive \
             -lpthread -Wl,--no-whole-archive",
            Os::Linux,
            Stack::Unknown,
            128 * KB,
            "the compile command is from Kattis' C++ page; the stack and the 128 KB source limit \
             are unconfirmed",
        ),
        profile(
            "dmoj",
            "C++17 (g++)",
            "g++ -Wall -O2 -lm -march=native -std=c++17 -DONLINE_JUDGE -s",
            Os::Linux,
            Stack::Unknown,
            64 * KB,
            "the command is from DMOJ's judge-server C++ executor; the g++ version, the stack and \
             the 64 KB source limit are a site's own",
        ),
    ]
}

/// The built-in profile called `name`.
pub fn builtin(name: &str) -> Option<Profile> {
    builtins().into_iter().find(|p| p.name == name)
}

/// The judge a file names with `// judge: <name>`.
pub fn named_in(source: &str) -> Option<&str> {
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix("// judge:"))
        .map(str::trim)
}

/// The judge a build of `path` (whose text is `source`) is for. `chosen`
/// is the name given on the command line.
pub fn resolve(path: &Path, source: &str, chosen: Option<&str>) -> Result<Profile, String> {
    let project = project(path);
    let config = project_config(path)?;
    let (name, from) = match (chosen, named_in(source), &config.default) {
        (Some(name), _, _) => (name.to_string(), "`--judge`".to_string()),
        (None, Some(name), _) => (name.to_string(), "the `// judge:` line".to_string()),
        (None, None, Some(name)) => {
            let file = project.as_ref().map(|(f, _)| f.display().to_string()).unwrap_or_default();
            (name.clone(), format!("`{file}`"))
        }
        (None, None, None) => ("generic".to_string(), String::new()),
    };
    config.profile(&name).ok_or_else(|| {
        let known: Vec<String> = config.names();
        format!("{from} names judge `{name}`, which is not one of: {}", known.join(", "))
    })
}

/// The `[judge]` part of the nearest `cp.toml` above `path`.
pub fn project_config(path: &Path) -> Result<Config, String> {
    match project(path) {
        Some((file, text)) => Config::parse(&text).map_err(|e| format!("{}: {e}", file.display())),
        None => Ok(Config::default()),
    }
}

/// The nearest `cp.toml` above `path`, and its text.
fn project(path: &Path) -> Option<(std::path::PathBuf, String)> {
    let start = path.parent()?;
    let start = if start.as_os_str().is_empty() { Path::new(".") } else { start };
    start.ancestors().find_map(|dir| {
        let file = dir.join("cp.toml");
        std::fs::read_to_string(&file).ok().map(|text| (file, text))
    })
}

/// The `[judge]` part of a `cp.toml`.
#[derive(Default, Debug)]
pub struct Config {
    pub default: Option<String>,
    /// Each `[judge.<name>]` table, in order: its name and its keys.
    pub tables: Vec<(String, Vec<(String, Value)>)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Int(u64),
    Bool(bool),
    List(Vec<String>),
}

impl Config {
    /// Reads `[judge]` and `[judge.<name>]`; other tables are not ours.
    pub fn parse(text: &str) -> Result<Config, String> {
        let mut config = Config::default();
        let mut table: Option<Option<String>> = None;
        for (at, line) in text.lines().enumerate() {
            let line = strip_comment(line).trim();
            if line.is_empty() {
                continue;
            }
            let err = |message: String| format!("line {}: {message}", at + 1);
            if let Some(head) = line.strip_prefix('[') {
                let head = head.strip_suffix(']').ok_or_else(|| err("a table header ends with `]`".into()))?;
                let head = head.trim();
                table = if head == "judge" {
                    Some(None)
                } else if let Some(name) = head.strip_prefix("judge.") {
                    let name = name.trim().trim_matches('"').to_string();
                    config.tables.push((name.clone(), Vec::new()));
                    Some(Some(name))
                } else {
                    None
                };
                continue;
            }
            let Some(table) = &table else { continue };
            let (key, value) = line.split_once('=').ok_or_else(|| err(format!("`{line}` is not `key = value`")))?;
            let key = key.trim().to_string();
            let value = parse_value(value.trim()).map_err(&err)?;
            match table {
                None if key == "judge" => match value {
                    Value::Str(name) => config.default = Some(name),
                    _ => return Err(err("`judge` is a profile's name in quotes".into())),
                },
                None => return Err(err(format!("`[judge]` has `judge = \"<name>\"`, not `{key}`"))),
                Some(_) => {
                    check_key(&key, &value).map_err(&err)?;
                    config.tables.last_mut().unwrap().1.push((key, value));
                }
            }
        }
        for (name, keys) in &config.tables {
            if builtin(name).is_none() && !keys.iter().any(|(k, _)| k == "extends") {
                return Err(format!(
                    "`[judge.{name}]` is not a built-in profile, so it says which one it `extends`"
                ));
            }
        }
        Ok(config)
    }

    /// Every profile's name: the built-in ones, then the project's own.
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = builtins().into_iter().map(|p| p.name).collect();
        for (name, _) in &self.tables {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
        names
    }

    /// The profile called `name`, with the project's changes.
    pub fn profile(&self, name: &str) -> Option<Profile> {
        self.profile_within(name, 0)
    }

    fn profile_within(&self, name: &str, depth: usize) -> Option<Profile> {
        if depth > 16 {
            return None;
        }
        let keys: Vec<&(String, Value)> =
            self.tables.iter().filter(|(n, _)| n == name).flat_map(|(_, keys)| keys).collect();
        let base = keys.iter().find_map(|(k, v)| match (k.as_str(), v) {
            ("extends", Value::Str(base)) if base != name => Some(base.clone()),
            _ => None,
        });
        let mut profile = match base {
            Some(base) => self.profile_within(&base, depth + 1)?,
            None => builtin(name)?,
        };
        if profile.name != name {
            profile.name = name.to_string();
            profile.evidence = format!("this project's `cp.toml`, from `{}`", profile.evidence);
        } else if !keys.is_empty() {
            profile.evidence = format!("changed by this project's `cp.toml`; {}", profile.evidence);
        }
        for (key, value) in keys {
            apply(&mut profile, key, value);
        }
        Some(profile)
    }
}

const KEYS: &[(&str, &str)] = &[
    ("extends", "a profile's name in quotes"),
    ("title", "text in quotes"),
    ("compiler", "a command in quotes"),
    ("flags", "a list of flags in quotes"),
    ("os", "\"linux\" or \"windows\""),
    ("stack_mb", "a whole number of MB, or \"unlimited\""),
    ("source_kb", "a whole number of KB"),
    ("int128", "true or false"),
];

fn check_key(key: &str, value: &Value) -> Result<(), String> {
    let Some((_, wants)) = KEYS.iter().find(|(k, _)| *k == key) else {
        let keys: Vec<&str> = KEYS.iter().map(|(k, _)| *k).collect();
        return Err(format!("a judge has no `{key}`; its keys are {}", keys.join(", ")));
    };
    let fits = matches!(
        (key, value),
        ("extends" | "title" | "compiler", Value::Str(_))
            | ("flags", Value::List(_))
            | ("os", Value::Str(_))
            | ("stack_mb", Value::Int(_))
            | ("source_kb", Value::Int(_))
            | ("int128", Value::Bool(_))
    ) || matches!((key, value), ("stack_mb", Value::Str(s)) if s == "unlimited");
    let fits = fits && !matches!((key, value), ("os", Value::Str(s)) if s != "linux" && s != "windows");
    match fits {
        true => Ok(()),
        false => Err(format!("`{key}` is {wants}")),
    }
}

fn apply(profile: &mut Profile, key: &str, value: &Value) {
    match (key, value) {
        ("title", Value::Str(s)) => profile.title = s.clone(),
        ("compiler", Value::Str(s)) => profile.compiler = s.clone(),
        ("flags", Value::List(flags)) => profile.flags = flags.clone(),
        ("os", Value::Str(s)) => profile.os = if s == "windows" { Os::Windows } else { Os::Linux },
        ("stack_mb", Value::Int(mb)) => profile.stack = Stack::Bytes(mb << 20),
        ("stack_mb", Value::Str(_)) => profile.stack = Stack::Unlimited,
        ("source_kb", Value::Int(kb)) => profile.source_limit = (*kb as usize) * 1024,
        ("int128", Value::Bool(b)) => profile.int128 = *b,
        _ => {}
    }
}

fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (at, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..at],
            _ => {}
        }
    }
    line
}

fn parse_value(text: &str) -> Result<Value, String> {
    if let Some(inner) = text.strip_prefix('[') {
        let inner = inner.strip_suffix(']').ok_or("a list is `[\"a\", \"b\"]` on one line")?;
        let mut items = Vec::new();
        for item in inner.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            match parse_value(item)? {
                Value::Str(s) => items.push(s),
                _ => return Err("a list holds strings in quotes".into()),
            }
        }
        return Ok(Value::List(items));
    }
    if let Some(inner) = text.strip_prefix('"') {
        let inner = inner.strip_suffix('"').ok_or("a string ends with `\"`")?;
        return Ok(Value::Str(inner.to_string()));
    }
    match text {
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    text.replace('_', "")
        .parse()
        .map(Value::Int)
        .map_err(|_| format!("`{text}` is not a string, a number, true, false or a list"))
}

#[cfg(test)]
mod tests;
