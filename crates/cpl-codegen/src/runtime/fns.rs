//! Fn values: the empty fn, recursive lambdas and values found on first use.

use super::Piece;

pub(super) const PIECES: &[Piece] = &[
    Piece {
        name: "no_fn",
        needs: &["fail"],
        code: "\
// The fn field of a struct the program never builds: no value of the
// struct exists, so it is never called.
template <class R> struct NoFn {
    template <class... A> [[noreturn]] R operator()(A&&...) const { fail(\"a fn field of a struct that is never built was called\"); }
    template <class... A> [[noreturn]] static R never(A...) { fail(\"a fn field of a struct that is never built was called\"); }
    template <class... A> using Ptr = R (*)(A...);
    template <class... A> operator Ptr<A...>() const { return &never<A...>; }
};
",
    },
    Piece {
        name: "rec",
        needs: &[],
        code: "\
// A recursive local closure, passed itself first.
template <class F> struct Rec { F f; template <class... A> decltype(auto) operator()(A&&... a) { return f(*this, (A&&)a...); } };
template <class F> Rec<F> rec(F f) { return {f}; }
",
    },
    Piece {
        name: "lazy",
        needs: &[],
        code: "\
// A row a loop reaches by indices it never changes, found on first use.
template <class F> struct Lazy { F f; decltype(f()) p = 0; auto& operator()() { return *(p ? p : p = f()); } };
template <class F> Lazy<F> lazy(F f) { return {f}; }
",
    },
];
