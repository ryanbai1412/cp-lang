//! How lowering names a type in a message.

use super::*;

impl<'a> Lowerer<'a> {
    /// `ty` as the user would write it; a declared type being checked is
    /// shown as its source writes it, `[n]Int` rather than `[Int]Int`.
    pub(super) fn show(&self, ty: &Ty) -> String {
        if let Some(op) = self.op_of_type(ty) {
            let t = if matches!(op, super::mono::BareOp::And | super::mono::BareOp::Or) {
                Ty::Bool
            } else {
                Ty::Int
            };
            return self.show(&Ty::Fn(vec![t.clone(), t.clone()], Box::new(t)));
        }
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
                    GenArg::OptRange(_, _) => "?Int".to_string(),
                    GenArg::Domain(d) => domains[*d].clone(),
                    GenArg::Ty(ty) | GenArg::Sized(ty, _) | GenArg::Drawn(ty, _) => {
                        crate::show::ty_name(ty, domains, &structs, &self.enums)
                    }
                    GenArg::RuntimeInt(_) => "_".to_string(),
                    GenArg::Op(op) => format!("{op:?}"),
                })
                .collect();
            let written = cpl_check::written_name(&decl.name.text);
            structs[sid].name = format!("{written}<{}>", args.join(", "));
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

    /// A module's type is shown by the name the program writes, never the
    /// `m1_Pt` it is lowered to (R08.1.6: a tuple is not a `Pt`).
    #[test]
    fn a_module_type_is_named_as_written() {
        let pt = "use std.geometry.{Pt}\n";
        let cases = [
            (
                format!("{pt}let p: Pt = (1, 2)\nprint(p.x)\n"),
                "expected `Pt`, found `(Int, Int)`",
            ),
            (
                format!("{pt}let p = Pt(1, 2)\nlet s: Str = p\nprint(s)\n"),
                "expected `Str`, found `Pt`",
            ),
        ];
        for (source, want) in cases {
            let errors = errors(&source);
            assert!(errors.iter().any(|e| e == want), "{want}: {errors:?}");
            for e in &errors {
                assert!(!mangled(e), "a lowered name in {e:?}");
            }
        }
    }

    fn mangled(text: &str) -> bool {
        let b = text.as_bytes();
        (1..b.len()).any(|i| {
            let start = b[i - 1] == b'm' && (i < 2 || !b[i - 2].is_ascii_alphanumeric());
            let digits = b[i..].iter().take_while(|c| c.is_ascii_digit()).count();
            start && digits > 0 && b.get(i + digits) == Some(&b'_')
        })
    }
}
