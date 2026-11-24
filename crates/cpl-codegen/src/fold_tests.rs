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

#[test]
fn self_recursive_instances_call_their_own_template() {
    let cpp = "namespace prog {\n\
        struct A{long long v;};\n\
        struct B{long long v;};\n\
        inline long long A_depth(const A&a,long long n){\n\
        if(n==0)return a.v;\n\
        return A_depth(a,n-1)+3;\n\
        }\n\
        inline long long B_depth(const B&a,long long n){\n\
        if(n==0)return a.v;\n\
        return B_depth(a,n-1)+5;\n\
        }\n\
        long long both(){return A_depth(A{1},2)+B_depth(B{2},3);}\n\
        }\n";
    let origins: BTreeMap<String, String> =
        [("A_depth", "depth"), ("B_depth", "depth"), ("both", "both")]
            .into_iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
    let folded = fold(cpp, &origins);
    assert_eq!(folded.members.len(), 2, "{}", folded.cpp);
    assert_eq!(folded.cpp.matches("template<").count(), 2, "{}", folded.cpp);
    assert!(!folded.cpp.contains("B_depth"), "{}", folded.cpp);
    let open = folded.cpp.find("{\nif(n==0)").expect(&folded.cpp);
    let body = &folded.cpp[open..];
    assert!(body.contains("A_depth<T0,V1>(a,n-1)"), "{}", folded.cpp);
    assert!(
        folded
            .cpp
            .contains("A_depth<A,3>(A{1},2)+A_depth<B,5>(B{2},3)"),
        "{}",
        folded.cpp
    );
}

#[test]
fn instances_differing_in_whole_types_fold_on_those_types() {
    let pad = "long long pad=123456789;pad+=987654321;pad*=pad;pad-=pad;(void)pad;";
    let file = |get: &str| {
        format!(
            "namespace prog {{\n\
             struct P{{ll a;}};\n\
             inline ll Int_id(ll x){{\n\
             {pad}\n\
             return x;\n\
             }}\n\
             inline std::tuple<ll,P> P_id(const std::tuple<ll,P>& x){{\n\
             {pad}\n\
             return {get};\n\
             }}\n\
             ll both(){{return Int_id(3)+std::get<1>(P_id(std::tuple<ll,P>{{1,P{{2}}}})).a;}}\n\
             }}\n"
        )
    };
    let origins: BTreeMap<String, String> = [("Int_id", "id"), ("P_id", "id"), ("both", "both")]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    assert!(fold(&file("(x)"), &origins).members.is_empty());
    let folded = fold(&file("x"), &origins);
    let cpp = &folded.cpp;
    assert_eq!(folded.members.len(), 2, "{cpp}");
    assert!(cpp.contains("template<class T0,class T1>"), "{cpp}");
    assert!(cpp.contains("T0 Int_id(T1 x)"), "{cpp}");
    assert!(cpp.contains("Int_id<ll,ll>(3)"), "{cpp}");
    assert!(
        cpp.contains("Int_id<std::tuple<ll,P>,const std::tuple<ll,P>&>(std::tuple<ll,P>{1,P{2}})"),
        "{cpp}"
    );
}

#[test]
fn literals_that_fold_as_integers_include_hex_and_bools() {
    let o = origins();
    let base = file("+");
    let hex = base
        .replace("if(r>=7)r-=7;", "if(r>=0x1e)r-=0x1e;")
        .replace("if(r>=11)r-=11;", "if(r>=0x2e)r-=0x2e;");
    let folded = fold(&hex, &o);
    assert_eq!(folded.members.len(), 2, "{}", folded.cpp);
    assert!(
        folded.cpp.contains("Mod_7_1_add<Mod_7_1,0x1e>("),
        "{}",
        folded.cpp
    );
    let flag = |r: &str| format!("long long r=a.v+b.v;\nif(r>={r})");
    let bools = base
        .replace(&flag("7"), "long long r=a.v+b.v;bool t=true;\nif(r>=7)")
        .replace(&flag("11"), "long long r=a.v+b.v;bool t=false;\nif(r>=11)");
    let folded = fold(&bools, &o);
    assert!(folded.cpp.contains("bool t=V1;"), "{}", folded.cpp);
    assert!(
        folded.cpp.contains("Mod_7_1_add<Mod_7_1,true,7>("),
        "{}",
        folded.cpp
    );
    assert!(
        folded.cpp.contains("Mod_7_1_add<Mod_11_2,false,11>("),
        "{}",
        folded.cpp
    );
}

#[test]
fn a_generic_fn_with_one_instance_is_left_as_written() {
    let mut o = origins();
    o.insert("Mod_11_2_add".into(), "Mod.sub".into());
    let cpp = file("+");
    let folded = fold(&cpp, &o);
    assert!(folded.members.is_empty(), "{}", folded.cpp);
    assert_eq!(folded.cpp, cpp);
}

#[test]
fn instances_differing_in_a_plain_enum_fold_on_it() {
    let cpp = "namespace prog {\nenum E1{a1,b1};\nenum E2{a2,b2};\n\
        inline long long E1_f(E1 e){\nreturn (long long)e+1;\n}\n\
        inline long long E2_f(E2 e){\nreturn (long long)e+1;\n}\n\
        long long both(){return E1_f(a1)+E2_f(b2);}\n}\n";
    let o: BTreeMap<String, String> = [("E1_f", "f"), ("E2_f", "f"), ("both", "both")]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    let folded = fold(cpp, &o);
    assert_eq!(folded.members.len(), 2, "{}", folded.cpp);
    assert!(
        folded.cpp.contains("E1_f<E1>(a1)+E1_f<E2>(b2)"),
        "{}",
        folded.cpp
    );
}

#[test]
fn a_template_calling_another_passes_on_each_of_its_parameters() {
    let cpp = file("+").replace(
        "long long both(){",
        "inline long long G7(){\nreturn Mod_7_1_add(Mod_7_1{3},Mod_7_1{5}).v;\n}\n\
         inline long long G11(){\nreturn Mod_11_2_add(Mod_11_2{3},Mod_11_2{5}).v;\n}\n\
         long long both(){",
    );
    let mut o = origins();
    o.insert("G7".into(), "g".into());
    o.insert("G11".into(), "g".into());
    let folded = fold(&cpp, &o);
    assert!(
        folded
            .cpp
            .contains("return Mod_7_1_add<T0,V1>(T0{3},T0{5}).v;"),
        "{}",
        folded.cpp
    );
}

#[test]
fn template_parameters_avoid_a_name_the_file_uses() {
    let cpp = file("+").replace("long long r=", "long long T0=1;(void)T0;long long r=");
    let folded = fold(&cpp, &origins());
    assert!(
        folded.cpp.contains("template<class T_0,auto V1>"),
        "{}",
        folded.cpp
    );
}

#[test]
fn a_type_or_integer_that_names_or_splits_something_never_folds() {
    let o = origins();
    let base = file("+");
    let scoped = base
        .replace(
            "long long r=a.v+b.v;\nif(r>=7)",
            "long long r=a.v+b.v+Mod_7_1::k;\nif(r>=7)",
        )
        .replace(
            "long long r=a.v+b.v;\nif(r>=11)",
            "long long r=a.v+b.v+Mod_11_2::k;\nif(r>=11)",
        );
    let tagged = base
        .replace(
            "long long r=a.v+b.v;\nif(r>=7)",
            "struct Mod_7_1 c=a;long long r=c.v+b.v;\nif(r>=7)",
        )
        .replace(
            "long long r=a.v+b.v;\nif(r>=11)",
            "struct Mod_11_2 c=a;long long r=c.v+b.v;\nif(r>=11)",
        );
    let fraction = base
        .replace(
            "long long r=a.v+b.v;\nif(r>=7)",
            "long long r=a.v+b.v+(long long)1.5;\nif(r>=7)",
        )
        .replace(
            "long long r=a.v+b.v;\nif(r>=11)",
            "long long r=a.v+b.v+(long long)1.7;\nif(r>=11)",
        );
    for cpp in [scoped, tagged, fraction] {
        let folded = fold(&cpp, &o);
        assert!(folded.members.is_empty(), "{}", folded.cpp);
        assert_eq!(folded.cpp, cpp);
    }
}

#[test]
fn a_call_of_a_folded_instance_is_rewritten_word_by_word() {
    let calls: BTreeMap<String, String> = [("f_1", "f<A,1>"), ("h_1", "h2")]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    let called = |text: &str| crate::fold::called(text, &calls);
    assert_eq!(called("x=f_1(a)+g_1(b)+xf_1;"), "x=f<A,1>(a)+g_1(b)+xf_1;");
    assert_eq!(
        called("y=f_1>=z;q=f_1;"),
        "y=f<A,1> >=z;q=f_1;".replacen("q=f_1", "q=f<A,1>", 1)
    );
    assert_eq!(called("v=f_1>w"), "v=f<A,1> >w");
    assert_eq!(called("h_1>=0"), "h2>=0");
    assert_eq!(crate::fold::called("f_1", &BTreeMap::new()), "f_1");
}

#[test]
fn a_whole_type_is_a_known_name_with_its_arguments_and_refs_on_one_line() {
    let spans = |cpp: &str| crate::fold::whole_types(cpp, &["ll", "P", "Mod", "p_t"]);
    assert_eq!(
        spans("std::tuple<ll,P> f(const std::tuple<ll,P>& x, P& a, P* b, P** c, const P d);\n"),
        [
            "std::tuple<ll,P>",
            "const std::tuple<ll,P>&",
            "P&",
            "P*",
            "P**",
            "const P"
        ]
    );
    assert_eq!(
        spans("cpl::Mod<7> g(cpl::Other<ll> y, std::vector<p_t> z);\n"),
        ["cpl::Mod<7>", "std::vector<p_t>"]
    );
    let none: [&str; 0] = [];
    assert_eq!(
        spans("P q, ll r, prog::P& z, a::std::vector<ll> v, std::less<ll> c;\n"),
        none
    );
    assert_eq!(
        spans("ll h(std::vector<ll>::iterator i, std::array<ll,3+1> a);\n"),
        none
    );
    assert_eq!(spans("ll n(x:const P&y);\n"), ["const P&"]);
    assert_eq!(spans("ll u(Q& a, const Q b, Q* c, std::Q<ll> d);\n"), none);
    assert_eq!(spans("ll k(std::vector<ll,\nll> v);\n"), none);
}

#[test]
fn a_folded_fn_is_declared_by_its_own_signature_before_its_first_use() {
    let cpp = "namespace prog {\n\
         inline long long A_1(long long n){\nreturn n?A_1(n-1):0;\n}\n\
         inline long long A_2(long long n){\nreturn n?A_2(n-1):0;\n}\n\
         long long both(){return A_1(3)+A_2(4);}\n\
         }\n";
    let o: BTreeMap<String, String> = [("A_1", "A"), ("A_2", "A"), ("both", "both")]
        .into_iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();
    assert_eq!(
        fold(cpp, &o).cpp,
        "namespace prog {\ninline long long A_1(long long n);\n\
         inline long long A_1(long long n){\nreturn n?A_1(n-1):0;\n}\n\
         long long both(){return A_1(3)+A_1(4);}\n}\n"
    );
    let whole = fold(&file("+"), &origins());
    assert_eq!(
        whole.members["Mod_11_2_add"].as_deref(),
        Some(
            "template<class T0,auto V1>\ninline T0 Mod_7_1_add(const T0&a,const T0&b){\n\
             long long r=a.v+b.v;\nif(r>=V1)r-=V1;\nreturn T0{r};\n}\n"
        )
    );
    assert_eq!(whole.members["Mod_7_1_add"], None);
    let folded = whole.cpp;
    assert!(
        folded.contains(
            "\ntemplate<class T0,auto V1>inline T0 Mod_7_1_add(const T0&a,const T0&b);\n\
             template<class T0,auto V1>\ninline T0 Mod_7_1_add(const T0&a,const T0&b){\n"
        ),
        "{folded}"
    );
    let spaced = file("+")
        .replace(
            "(const Mod_7_1&a,const Mod_7_1&b)",
            "( const Mod_7_1 & a, const Mod_7_1&b )",
        )
        .replace(
            "(const Mod_11_2&a,const Mod_11_2&b)",
            "( const Mod_11_2 & a, const Mod_11_2&b )",
        );
    let folded = fold(&spaced, &origins()).cpp;
    assert!(
        folded.contains(
            "\ntemplate<class T0,auto V1>inline T0 Mod_7_1_add(const T0&a,const T0&b);\n\
             template<class T0,auto V1>\ninline T0 Mod_7_1_add( const T0 & a, const T0&b ){\n"
        ),
        "{folded}"
    );
}
