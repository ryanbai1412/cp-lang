//! Pass one of collecting declarations, per form: the form gets its id and its name in its
//! module, and fills the roles it is marked with.
use super::collect::{again, misplaced_role, once, operator_method, unimplemented};
use super::forms::{AttrForm, StructForm, TraitForm, TyForm};
use super::*;
use crate::attrs::Decorator;
use cpl_next_ir::ids::ItemId;
use cpl_next_syntax::ast::AstNode as _;

/// Where decorator `x` is written, or `at` for one the compiler made.
pub(super) fn attr_at(x: &AttrForm, at: Span) -> Span {
    x.node.as_ref().map_or(at, |n| n.syntax().span())
}

/// The roles a form is marked with: each `@role(..)` and the role it names, if it names one.
fn roles(attrs: &[AttrForm]) -> impl Iterator<Item = (&AttrForm, Option<Role>)> {
    (attrs.iter())
        .filter(|x| x.attr == Decorator::Role)
        .map(|x| (x, form_role(x)))
}

/// Struct `sd`: its id, its name among `own`, and the roles and tables it fills.
pub(super) fn name_struct(
    d: &mut Decls,
    own: &mut HashMap<String, Named>,
    mid: ModId,
    item: &ItemId,
    sd: &StructForm,
) -> StructId {
    let id = StructId(d.structs.len());
    let n = sd.generics.len();
    let name = d.text(sd.name).to_string();
    // One of a name and another arity is a sibling of the first; one of the same arity is a
    // duplicate (R02.11.4).
    match own.get(&name) {
        Some(&Named::Struct(f)) => {
            let first = (f, d.structs[f.0].generics.len());
            let of_name =
                (Arc::make_mut(&mut d.struct_arity).entry(f)).or_insert_with(|| vec![first]);
            match of_name.iter().any(|(_, k)| *k == n) {
                true => d.errors.push(Msg::at(
                    item.at(sd.span),
                    Msg::verdict(
                        cpl_diag::codes::DUPLICATE_DEFINITION,
                        format!("a second struct `{name}` with {n} type parameters (R02.11.4)"),
                    ),
                )),
                false => of_name.push((id, n)),
            }
        }
        _ => _ = own.insert(name.clone(), Named::Struct(id)),
    }
    for (x, role) in roles(&sd.attrs) {
        // The tuple role takes one struct per field count (R02.5.1).
        if role == Some(Role::Tuple) {
            let n = sd.fields.len();
            match d.tuples.contains_key(&n) {
                true => d.errors.push(again(item, sd.span, "role(tuple)")),
                false => _ = Arc::make_mut(&mut d.tuples).insert(n, id),
            }
            continue;
        }
        let slot = match role {
            Some(Role::Option) => &mut d.roles.option,
            Some(Role::Recipe) => &mut d.roles.recipe,
            Some(Role::Cuts) => &mut d.roles.cut,
            Some(Role::List) => &mut d.sugar.list,
            Some(Role::Str) => &mut d.sugar.str,
            Some(Role::Map) => &mut d.sugar.map,
            Some(Role::Array) => &mut d.sugar.array,
            Some(Role::Pack) => &mut d.roles.pack,
            _ => {
                let at = item.at(attr_at(x, sd.span));
                d.errors.push(misplaced_role(item, at, role, "struct"));
                continue;
            }
        };
        let dup = again(
            item,
            sd.span,
            &format!("role({})", role.map_or("", Role::name)),
        );
        once(slot, id, dup, &mut d.errors);
    }
    if sd.record {
        let names = sd
            .fields
            .iter()
            .map(|f| d.text(f.name).to_string())
            .collect();
        Arc::make_mut(&mut d.records).insert(names, id);
    }
    let member = sd.attrs.iter().find(|x| x.attr == Decorator::Member);
    if let Some(m) = member.and_then(AttrForm::name_arg) {
        let m = d.syms.intern(&m);
        Arc::make_mut(&mut d.member_types).insert(m, id);
    }
    let mut consts = HashMap::new();
    for (c, ty, init) in &sd.consts {
        consts.insert(*c, ConstId(d.consts.len()));
        let ty = ty.as_ref().map(|t| match t {
            TyForm::Written(t) => t.clone(),
            _ => unreachable!("a const's type is written"),
        });
        d.consts.push(LibConst {
            module: mid,
            name: format!("{name}.{}", d.text(*c)),
            item: item.clone(),
            ty,
            init: init.clone(),
        });
    }
    d.structs.push(StructDecl {
        name,
        record: sd.record,
        generics: sd.generics.iter().map(|g| d.text(*g).to_string()).collect(),
        fields: Vec::new(),
        var_fields: Vec::new(),
        pub_fields: Vec::new(),
        cached: Vec::new(),
        field_at: Vec::new(),
        defaults: Vec::new(),
        module: mid,
        item: item.clone(),
        n_sets: 0,
        methods: HashMap::new(),
        consts,
        shaped: HashMap::new(),
        ctor: None,
        variants: Vec::new(),
        errors: Vec::new(),
        values: Vec::new(),
        bounds: Vec::new(),
    });
    id
}

/// Trait `t`: its id, its name among `own`, the roles it fills, and the operator or literal it
/// is the trait of. `op_methods` holds each operator trait's method.
pub(super) fn name_trait(
    d: &mut Decls,
    own: &mut HashMap<String, Named>,
    op_methods: &mut HashMap<TraitId, Sym>,
    mid: ModId,
    item: &ItemId,
    t: &TraitForm,
) -> TraitId {
    let id = TraitId(d.traits.len());
    for (x, role) in roles(&t.attrs) {
        let slot = match role {
            Some(Role::Iteration) => &mut d.roles.iteration,
            Some(Role::Positions) => &mut d.roles.domain,
            Some(Role::Order) => &mut d.roles.order,
            Some(Role::Convert) => &mut d.roles.convert,
            Some(Role::Counted) => &mut d.roles.counted,
            Some(Role::RangeBounds) => &mut d.roles.bounds,
            Some(Role::Endpoint) => &mut d.roles.endpoint,
            Some(Role::DeriveFields) => &mut d.roles.derive_fields,
            _ => {
                let at = item.at(attr_at(x, t.span));
                d.errors.push(misplaced_role(item, at, role, "trait"));
                continue;
            }
        };
        let dup = again(item, t.span, "role(..)");
        once(slot, id, dup, &mut d.errors);
    }
    let lit = t.attrs.iter().find(|x| x.attr == Decorator::Literal);
    if let Some(text) = lit.and_then(AttrForm::str_arg) {
        let text = d.syms.intern(&text);
        match d.literals.iter().any(|(o, _)| *o == text) {
            true => d
                .errors
                .push(again(item, t.span, Decorator::Literal.name())),
            false => d.literals.push((text, id)),
        }
    }
    let op = t.attrs.iter().find(|x| x.attr == Decorator::Operator);
    if let Some(text) = op.and_then(AttrForm::str_arg) {
        let m = operator_method(t);
        let text = d.syms.intern(&text);
        let method = |tr: &TraitId| match op_methods.get(tr) {
            Some(m) => Some(*m),
            None => d.traits[tr.0].methods.keys().next().copied(),
        };
        match (d.operators.iter()).any(|(o, tr)| *o == text && method(tr) == m) {
            true => d
                .errors
                .push(again(item, t.span, Decorator::Operator.name())),
            false => {
                d.operators.push((text, id));
                op_methods.extend(m.map(|m| (id, m)));
            }
        }
    }
    let name = d.text(t.name).to_string();
    own.insert(name.clone(), Named::Trait(id));
    let unimplemented = unimplemented(item, &t.attrs, &mut d.errors);
    d.traits.push(TraitDecl {
        name,
        module: mid,
        alias: t.alias,
        supers: Vec::new(),
        methods: HashMap::new(),
        errors: Vec::new(),
        defaults: Vec::new(),
        assoc: Vec::new(),
        assoc_bounds: Vec::new(),
        unimplemented,
        row: t.record,
    });
    id
}
