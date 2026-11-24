//! How lowering names a type in a message.

use super::*;

impl<'a> Lowerer<'a> {
    /// `ty` as the user would write it; a declared type being checked is
    /// shown as its source writes it, `[n]Int` rather than `[Int]Int`.
    pub(super) fn show(&self, ty: &Ty) -> String {
        if let Some((_, text)) = self.written.iter().rev().find(|(t, _)| t == ty) {
            return text.clone();
        }
        let domains: Vec<String> = (0..self.domains.len())
            .map(|d| self.domain_said(d))
            .collect();
        let structs = self.shown_structs(&domains);
        crate::show::ty_name(ty, &domains, &structs, &self.enums)
    }

    /// The structs, each named as it is written: a struct made from a
    /// generic one is `Mod<998244353>`, not the name its copy has in the C++.
    pub(super) fn shown_structs(&self, domains: &[String]) -> Vec<StructDef> {
        let mut structs: Vec<StructDef> = self.structs.iter().map(|s| s.def.clone()).collect();
        for sid in 0..structs.len() {
            let (Some(decl), Some((_, args))) =
                (self.structs[sid].decl, &self.structs[sid].origin_gen)
            else {
                continue;
            };
            let args: Vec<String> = args
                .iter()
                .map(|arg| match arg {
                    GenArg::Int(v) => v.to_string(),
                    GenArg::Range(_, _) => "Int".to_string(),
                    GenArg::Domain(d) => domains[*d].clone(),
                    GenArg::Ty(ty) => crate::show::ty_name(ty, domains, &structs, &self.enums),
                    GenArg::RuntimeInt(_) => "_".to_string(),
                    GenArg::Op(op) => format!("{op:?}"),
                })
                .collect();
            structs[sid].name = format!("{}<{}>", decl.name.text, args.join(", "));
        }
        structs
    }
}

#[cfg(test)]
mod tests {
    fn errors(source: &str) -> Vec<String> {
        let mut session = cpl_check::Session::new();
        session.set_file("t.cp", source);
        let analysis = session.analysis("t.cp").expect("set");
        match crate::lower(&analysis) {
            Ok(_) => Vec::new(),
            Err(diags) => diags.into_iter().map(|d| d.message).collect(),
        }
    }

    #[test]
    fn a_declared_counted_array_is_named_as_written() {
        for written in ["[n]Int", "[0..n]Int", "[1..=n]Int", "[2 * n]Int"] {
            let source = format!(
                "let n = read::<Int>()\nlet xs = [1, 2]\nlet d: {written} = [x for x in xs]\nprint(d)\n"
            );
            let errors = errors(&source);
            let want = format!("expected `{written}`, found `List<Int>`");
            assert!(errors.contains(&want), "{written}: {errors:?}");
        }
    }
}
