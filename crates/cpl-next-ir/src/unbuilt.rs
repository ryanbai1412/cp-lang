//! What the new pipeline refuses: each feature the spec has and this pipeline has not built,
//! one variant per kind. A refusal says nothing about the program, and the list only shrinks
//! (`next_refusal_kinds` counts the variants): building a feature deletes its variant.
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Unbuilt {
    // Patterns.
    PatternOnUnknownType,
    LiteralPattern {
        ty: String,
    },
    DomainPattern,
    LetPattern,
    // Generators, lenders and function values.
    GeneratorAsValue {
        name: String,
    },
    LambdaParam,
    NestedFn {
        what: String,
    },
    FnValueFixedSet,
    FnValueSetsDiffer,
    MutArgToFnValue,
    FnValueInVariant,
    LambdaNamesOuter,
    LambdaMakesLambda,
    // Tables, ranges and indexing.
    TableOverIndex,
    RangeOfEnds {
        ty: String,
    },
    WriteParamCell,
    SliceOpenEnd,
    ByOnWalk,
    RangeTypeOutsideBody,
    RangeIndexedArray,
    // Operators and literals.
    FloatOperator,
    OperatorTrait {
        ty: String,
    },
    Operator,
    FloatLiteral {
        text: String,
    },
    NonAsciiString,
    Unwrap {
        ty: String,
    },
    Coalesce {
        ty: String,
    },
    Membership,
    // Printing and reading.
    FormatSpec {
        ty: String,
    },
    HexFormat,
    Printing {
        ty: String,
    },
    PrintNamedArgs,
    Read {
        ty: String,
    },
    // Calls, conversions and traits.
    TypeArgsValue,
    AliasCall {
        name: String,
    },
    CoreStandInArity {
        name: String,
        given: usize,
        takes: usize,
    },
    Derived {
        tr: String,
        ty: String,
    },
    FieldwiseTrait {
        ty: String,
        tr: String,
    },
    ConversionWithoutConvert,
    TraitIntrinsic {
        op: String,
    },
    Intrinsic {
        name: String,
    },
    BoundNotNamed,
    DomainTypeArg {
        func: String,
        domain: String,
    },
    // Statements and control.
    IfLet {
        ty: String,
    },
    Condition,
    LetElse {
        ty: String,
    },
    Assignment,
    ForForm,
    ChecksValue,
    ChecksField,
    MultitestArg,
    // Items and modules.
    Decorator {
        name: String,
    },
    FnWithoutBody,
    UndeclaredTopDomain,
    DomainKind {
        kind: String,
    },
    TaggedVariant {
        name: String,
    },
    VariantValue {
        name: String,
    },
    LibraryOverload {
        name: String,
    },
    MethodsOfOneName {
        name: String,
    },
    OverloadByLabels,
    LazyParam {
        name: String,
    },
    UseWithoutNames {
        module: String,
    },
    UseOfOwn {
        name: String,
    },
    UseMissing {
        module: String,
        name: String,
    },
    // Forms the parser reads that no arm above names yet, by their syntax kind.
    Expression {
        kind: String,
    },
    Statement {
        kind: String,
    },
    Type {
        kind: String,
    },
    LibraryItem {
        kind: String,
    },
}

impl Unbuilt {
    /// The variant's name (`ForForm`): what `next_refusal_kinds` counts and `--json` reports.
    pub fn variant(&self) -> String {
        let debug = format!("{self:?}");
        let end = debug.find([' ', '{', '(']).unwrap_or(debug.len());
        debug[..end].to_string()
    }
}

impl fmt::Display for Unbuilt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Unbuilt::*;
        match self {
            PatternOnUnknownType => write!(f, "a pattern on a value of a type not known yet"),
            LiteralPattern { ty } => write!(f, "a literal pattern on a `{ty}`"),
            DomainPattern => write!(f, "a `domain` pattern"),
            LetPattern => write!(f, "unsupported pattern"),
            GeneratorAsValue { name } => write!(
                f,
                "`{name}` yields or lends, and is used other than in a `for` or as the target of an assignment"
            ),
            LambdaParam => write!(f, "unsupported lambda parameter"),
            NestedFn { what } => write!(f, "a nested fn with {what}"),
            FnValueFixedSet => write!(
                f,
                "this function value came from a call, so the functions it can hold are fixed there; a new one cannot be added here"
            ),
            FnValueSetsDiffer => write!(f, "these two function values have different fixed sets"),
            MutArgToFnValue => write!(f, "a `mut` argument to a function value is not built yet"),
            FnValueInVariant => write!(f, "a function value in a variant's payload"),
            LambdaNamesOuter => write!(f, "a lambda that names the lambda around it"),
            LambdaMakesLambda => write!(f, "a lambda that makes a lambda with its own parameters"),
            TableOverIndex => write!(f, "a table over this index"),
            RangeOfEnds { ty } => write!(f, "a range value with `{ty}` ends"),
            WriteParamCell => write!(f, "writing a cell of a type parameter"),
            SliceOpenEnd => write!(f, "a slice with an end left out needs core's `@index` fn of the ends"),
            ByOnWalk => write!(f, "`by` on a walk that is not a range"),
            RangeTypeOutsideBody => write!(f, "a range type whose ends are not constants, outside a body"),
            RangeIndexedArray => write!(f, "an array indexed by a range outside a `let` or `read`"),
            FloatOperator => write!(f, "this operator on `F64`"),
            OperatorTrait { ty } => write!(f, "an operator on `{ty}` (its operator trait)"),
            Operator => write!(f, "unsupported operator"),
            FloatLiteral { text } => write!(f, "the float literal `{text}`"),
            NonAsciiString => write!(f, "a string with a character outside ASCII"),
            Unwrap { ty } => write!(f, "`!` on a `{ty}`"),
            Coalesce { ty } => write!(f, "`??` on a `{ty}`"),
            Membership => write!(f, "comparison of this type is not supported"),
            FormatSpec { ty } => write!(f, "a format spec on `{ty}`"),
            HexFormat => write!(f, "a hex or binary format spec"),
            Printing { ty } => write!(f, "printing a `{ty}`"),
            PrintNamedArgs => write!(f, "named arguments to `print`"),
            Read { ty } => write!(f, "`read` of `{ty}`"),
            TypeArgsValue => write!(f, "a type with type arguments used as a value"),
            AliasCall { name } => write!(f, "building a value by calling the alias `{name}`"),
            CoreStandInArity { name, given, takes } => {
                write!(f, "`{name}` with {given} arguments (the core stand-in takes {takes})")
            }
            Derived { tr, ty } => write!(f, "derived `{tr}` for `{ty}`"),
            FieldwiseTrait { ty, tr } => write!(f, "`{ty}` meeting `{tr}` by its fields"),
            ConversionWithoutConvert => write!(f, "a conversion without core's `@convert` trait"),
            TraitIntrinsic { op } => write!(f, "intrinsic `{op}` of a trait"),
            Intrinsic { name } => write!(f, "intrinsic `{name}`"),
            BoundNotNamed => write!(f, "a bound that is not a named trait"),
            DomainTypeArg { func, domain } => write!(
                f,
                "`{func}` is called with its own domain `{domain}` as a type argument, so each call would need a new copy of it; declare the domain outside, or pass its values as `Int`"
            ),
            IfLet { ty } => write!(f, "`if let` on a `{ty}`"),
            Condition => write!(f, "unsupported condition"),
            LetElse { ty } => write!(f, "let-else on a `{ty}`"),
            Assignment => write!(f, "unsupported assignment"),
            ForForm => write!(f, "`for` takes `name in lo..hi`"),
            ChecksValue => write!(f, "a `checks` value other than `on` or `off`"),
            ChecksField => write!(f, "this `with checks(...)` field"),
            MultitestArg => write!(f, "an argument of `@!multitest` other than `until_eof`"),
            Decorator { name } => write!(f, "decorator `@{name}`"),
            FnWithoutBody => write!(f, "a fn with no body"),
            UndeclaredTopDomain => write!(f, "a top-level domain that is not declared"),
            DomainKind { kind } => write!(f, "a domain that is {kind}"),
            TaggedVariant { name } => write!(f, "a tagged variant `{name}`"),
            VariantValue { name } => write!(f, "the value of variant `{name}`"),
            LibraryOverload { name } => write!(f, "a library with two fns named `{name}`"),
            MethodsOfOneName { name } => write!(f, "methods of one name in one module (`{name}`)"),
            OverloadByLabels => write!(f, "fns of one name told apart by their labels"),
            LazyParam { name } => write!(f, "a lazy parameter `{name}`"),
            UseWithoutNames { module } => write!(f, "`use {module}` without names"),
            UseOfOwn { name } => write!(f, "`use` of `{name}`, which this file also declares"),
            UseMissing { module, name } => write!(f, "`{module}` has no pub `{name}` the slice declares"),
            Expression { kind } => write!(f, "expression {kind}"),
            Statement { kind } => write!(f, "statement {kind}"),
            Type { kind } => write!(f, "type {kind}"),
            LibraryItem { kind } => write!(f, "library item {kind}"),
        }
    }
}
