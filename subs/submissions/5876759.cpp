constexpr bool eq(int a, int b) {
    return !(a < b) && !(a > b);
}

struct DpValue {
    short dp_v, trans_steps;
};

constexpr DpValue max(DpValue a, DpValue b) {
    return a.dp_v > b.dp_v ? a : b;
}

template <int D>
struct Node {
    DpValue v;
    Node<D - 1> prev;
};

template <>
struct Node<0> {
    DpValue v;
};

template <int I>
constexpr DpValue solve_one(int ind,
                            const Node<I>& o,
                            short cur_steps,
                            DpValue opt) {
    if (get_item(I - 1) < get_item(ind)) {
        return solve_one(ind, o.prev, cur_steps + 1,
                         max(opt, DpValue {o.v.dp_v, cur_steps}));
    } else {
        return solve_one(ind, o.prev, cur_steps + 1, opt);
    }
}

template <>
constexpr DpValue solve_one<0>(int ind,
                               const Node<0>& o,
                               short cur_steps,
                               DpValue opt) {
    return {short(opt.dp_v + 1), opt.trans_steps};
}

template <int I>
void print_rec(const Node<I>& o, int steps) {
    if (!steps) {
        print_rec(o.prev, o.v.trans_steps - 1);
        print_num(get_item(I - 1));
    } else {
        print_rec(o.prev, steps - 1);
    }
}

template <>
void print_rec(const Node<0>&, int) {}

template <int D>
void print(const Node<D>& o, DpValue opt) {
    print_num(opt.dp_v);
    print_rec(o, opt.trans_steps);
}

template <int I>
constexpr DpValue solve_upd_opt(const Node<I>& o, DpValue opt) {
    return max(DpValue {o.v.dp_v, 0},
               DpValue {opt.dp_v, short(opt.trans_steps + 1)});
}

template <int D>
struct Ans {
    Node<D> o;
    DpValue opt;
};

template <int D>
void print(const Ans<D>& ans) {
    print(ans.o, ans.opt);
}

template <int I>
constexpr Ans<N> solve(const Node<I>& o, DpValue opt) {
    return solve<I + 1>({solve_one(I, o, 1, {0, I + 1}), o},
                        solve_upd_opt(o, opt));
}

template <>
constexpr Ans<500> solve<500>(const Node<500>& o, DpValue opt) {
    return {o, solve_upd_opt(o, opt)};
}

constexpr Ans<500> ans(solve(Node<0> {}, {0, 0}));

void solve() {
    print(ans);
}

int main() {
    solve();
}