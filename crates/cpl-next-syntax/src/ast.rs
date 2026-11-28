//! Typed views over the tree, rust-analyzer style. A view is a node of
//! one kind, and an accessor returns `None` when the child it reads is
//! missing or broken, so a parse error never reads as a valid node.

use crate::file::NodeKind;
use crate::token::TokenKind;
use crate::tree::{SyntaxNode, SyntaxToken};

pub trait AstNode: Sized {
    fn cast(node: SyntaxNode) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode;
}

macro_rules! nodes {
    ($($(#[$doc:meta])* $name:ident),* $(,)?) => {$(
        $(#[$doc])*
        #[derive(Clone, Debug)]
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            fn cast(node: SyntaxNode) -> Option<Self> {
                (node.kind() == NodeKind::$name).then_some(Self(node))
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    )*};
}

nodes! {
    /// One top-level item: its attached comments, then a declaration or a
    /// statement.
    Item,
    FnDecl,
    /// `fn (a: T) + (b: T)`: the name of an operator fn.
    OpFnName,
    StructDecl,
    EnumDecl,
    TraitDecl,
    ImplDecl,
    TypeAlias,
    DomainDecl,
    ModDecl,
    ExternDecl,
    UseDecl,
    LawDecl,
    /// The members of a struct, enum, trait or impl.
    Members,
}

/// A declaration an item can hold.
#[derive(Clone, Debug)]
pub enum Decl {
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Impl(ImplDecl),
    TypeAlias(TypeAlias),
    Domain(DomainDecl),
    Mod(ModDecl),
    Extern(ExternDecl),
    Use(UseDecl),
}

impl AstNode for Decl {
    fn cast(node: SyntaxNode) -> Option<Self> {
        Some(match node.kind() {
            NodeKind::FnDecl => Decl::Fn(FnDecl(node)),
            NodeKind::StructDecl => Decl::Struct(StructDecl(node)),
            NodeKind::EnumDecl => Decl::Enum(EnumDecl(node)),
            NodeKind::TraitDecl => Decl::Trait(TraitDecl(node)),
            NodeKind::ImplDecl => Decl::Impl(ImplDecl(node)),
            NodeKind::TypeAlias => Decl::TypeAlias(TypeAlias(node)),
            NodeKind::DomainDecl => Decl::Domain(DomainDecl(node)),
            NodeKind::ModDecl => Decl::Mod(ModDecl(node)),
            NodeKind::ExternDecl => Decl::Extern(ExternDecl(node)),
            NodeKind::UseDecl => Decl::Use(UseDecl(node)),
            _ => return None,
        })
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Decl::Fn(n) => n.syntax(),
            Decl::Struct(n) => n.syntax(),
            Decl::Enum(n) => n.syntax(),
            Decl::Trait(n) => n.syntax(),
            Decl::Impl(n) => n.syntax(),
            Decl::TypeAlias(n) => n.syntax(),
            Decl::Domain(n) => n.syntax(),
            Decl::Mod(n) => n.syntax(),
            Decl::Extern(n) => n.syntax(),
            Decl::Use(n) => n.syntax(),
        }
    }
}

/// The `Ident` of `node`'s `Name` child: `None` when the name is missing.
fn name_of(node: &SyntaxNode) -> Option<SyntaxToken> {
    node.children()
        .find(|n| n.kind() == NodeKind::Name)?
        .tokens()
        .find(|t| t.kind() == TokenKind::Ident)
}

impl Item {
    /// The declaration the item holds; `None` for a statement or a
    /// directive.
    pub fn decl(&self) -> Option<Decl> {
        self.0.children().find_map(Decl::cast)
    }

    /// The name the item declares. An impl, a `use`, an operator fn, a
    /// statement and a broken declaration have none.
    pub fn name(&self) -> Option<SyntaxToken> {
        name_of(self.decl()?.syntax())
    }
}

impl FnDecl {
    /// `None` for an operator fn, which [`FnDecl::op_name`] reads.
    pub fn name(&self) -> Option<SyntaxToken> {
        name_of(&self.0)
    }

    pub fn op_name(&self) -> Option<OpFnName> {
        self.0.children().find_map(OpFnName::cast)
    }
}

impl LawDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        name_of(&self.0)
    }
}

impl ImplDecl {
    /// The trait in `impl Trait for T`; `None` for an inherent `impl T`.
    pub fn trait_ref(&self) -> Option<SyntaxNode> {
        self.has_for().then(|| self.types().next()).flatten()
    }

    /// The type the impl is for: `T` in `impl T` and `impl Trait for T`.
    pub fn self_ty(&self) -> Option<SyntaxNode> {
        self.types().nth(usize::from(self.has_for()))
    }

    pub fn members(&self) -> Option<Members> {
        self.0.children().find_map(Members::cast)
    }

    fn has_for(&self) -> bool {
        self.0
            .tokens()
            .any(|t| t.kind() == TokenKind::For && t.parent() == self.0)
    }

    /// The trait and self type, in order: the header's nodes after the
    /// generics, before the where clause and members.
    fn types(&self) -> impl Iterator<Item = SyntaxNode> + '_ {
        self.0.children().filter(|n| {
            !matches!(
                n.kind(),
                NodeKind::GenericParams | NodeKind::WhereClause | NodeKind::Members
            )
        })
    }
}

impl Members {
    pub fn fns(&self) -> impl Iterator<Item = FnDecl> + '_ {
        self.0.children().filter_map(FnDecl::cast)
    }

    pub fn laws(&self) -> impl Iterator<Item = LawDecl> + '_ {
        self.0.children().filter_map(LawDecl::cast)
    }
}
