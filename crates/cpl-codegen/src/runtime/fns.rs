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
        name: "fn_ref",
        needs: &[],
        code: "\
// A recursive generator's callback, by reference and of one type, so its
// `cpl_each_` form is written once however deep the recursion goes.
template <class S> struct FnRef;
template <class R, class... A> struct FnRef<R(A...)> {
  void* o; R (*f)(void*, A...);
  template <class L, class = std::enable_if_t<!std::is_same_v<std::decay_t<L>, FnRef>>> FnRef(L& l) : o((void*)&l), f([](void* o, A... a) -> R { return (*(L*)o)((A)a...); }) {}
  R operator()(A... a) const { return f(o, (A)a...); }
};
template <class T> struct FnSig;
template <class L, class R, class... A> struct FnSig<R (L::*)(A...) const> { using type = R(A...); };
template <class L> FnRef<typename FnSig<decltype(&std::decay_t<L>::operator())>::type> fn_ref(L&& l) { return {l}; }
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
