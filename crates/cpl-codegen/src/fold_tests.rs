//! The instances of a generic fn that differ only in types and integers
//! become one C++ template (`crate::fold`).

use crate::fold::fold;
use std::collections::BTreeMap;

fn file(second_op: &str) -> String {
    format!(
        "namespace prog {{\n\
         struct Mod_7_1{{long long v;}};\n\
         struct Mod_11_2{{long long v;}};\n\
         inline Mod_7_1 Mod_7_1_add(const Mod_7_1&a,const Mod_7_1&b){{\n\
         long long r=a.v+b.v;\n\
         if(r>=7)r-=7;\n\
         return Mod_7_1{{r}};\n\
         }}\n\
         inline Mod_11_2 Mod_11_2_add(const Mod_11_2&a,const Mod_11_2&b){{\n\
         long long r=a.v{second_op}b.v;\n\
         if(r>=11)r-=11;\n\
         return Mod_11_2{{r}};\n\
         }}\n\
         long long both(){{return Mod_7_1_add(Mod_7_1{{3}},Mod_7_1{{5}}).v*100+Mod_11_2_add(Mod_11_2{{6}},Mod_11_2{{9}}).v;}}\n\
         }}\n"
    )
}

fn origins() -> BTreeMap<String, String> {
    [
        ("Mod_7_1_add", "Mod.add"),
        ("Mod_11_2_add", "Mod.add"),
        ("both", "both"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

#[test]
fn instances_differing_in_a_type_and_an_integer_become_one_template() {
    let folded = fold(&file("+"), &origins());
    let cpp = &folded.cpp;
    assert_eq!(folded.members.len(), 2, "{cpp}");
    assert_eq!(
        cpp.matches("template<class T0,auto V1>").count(),
        2,
        "{cpp}"
    );
    assert_eq!(cpp.matches("if(r>=").count(), 1, "{cpp}");
    assert!(cpp.contains("if(r>=V1)r-=V1;"), "{cpp}");
    assert!(cpp.contains("<Mod_7_1,7>(Mod_7_1{3}"), "{cpp}");
    assert!(cpp.contains("<Mod_11_2,11>(Mod_11_2{6}"), "{cpp}");
    assert!(!cpp.contains("Mod_11_2_add"), "{cpp}");
}

#[test]
fn instances_differing_in_anything_else_stay_apart() {
    let cpp = file("-");
    let folded = fold(&cpp, &origins());
    assert!(folded.members.is_empty());
    assert_eq!(folded.cpp, cpp);
}

#[test]
fn a_fn_with_a_preprocessor_line_inside_is_never_a_template() {
    let cpp = file("+").replace("if(r>=7)", "#ifdef LOCAL\n#endif\nif(r>=7)");
    let folded = fold(&cpp, &origins());
    assert!(folded.members.is_empty());
    assert_eq!(folded.cpp, cpp);
}

#[test]
fn folding_is_deterministic() {
    assert_eq!(
        fold(&file("+"), &origins()).cpp,
        fold(&file("+"), &origins()).cpp
    );
}
