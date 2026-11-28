//! Core's primitive types, by the names a program writes them.

/// A core primitive type (contract 9's `Type::prim`, which cpl-contract re-exports), each by the name a program writes: the
/// one list the checker reads core type names from. Std's types are not here; a tool reaches
/// them through `Type::decl` and `Type::implements`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Prim {
    Int,
    Bool,
    Char,
    U64,
    I128,
    F64,
}

impl Prim {
    pub const ALL: [Prim; 6] = [
        Prim::Int,
        Prim::Bool,
        Prim::Char,
        Prim::U64,
        Prim::I128,
        Prim::F64,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Prim::Int => "Int",
            Prim::Bool => "Bool",
            Prim::Char => "Char",
            Prim::U64 => "U64",
            Prim::I128 => "I128",
            Prim::F64 => "F64",
        }
    }

    /// The core type a program writes as `name`.
    pub fn named(name: &str) -> Option<Prim> {
        Prim::ALL.into_iter().find(|p| p.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::Prim;

    #[test]
    fn each_core_type_is_found_by_the_name_it_is_written() {
        for p in Prim::ALL {
            assert_eq!(Prim::named(p.name()), Some(p));
        }
        assert_eq!(Prim::named("Str"), None);
    }
}
