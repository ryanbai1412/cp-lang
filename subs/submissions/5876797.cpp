bool eq(int a, int b) {
    return !(a < b) && !(a > b);
}

short max(short a, short b) {
    return a > b ? a : b;
}

template <int D>
struct Node {
    short v;
    Node<D - 1> prev;
};

template <>
struct Node<0> {
    short v;
};

template <int I>
short solve_one(short ind, const Node<I>& o, short opt) {
    if (get_item(I - 1) < get_item(ind)) {
        return solve_one(ind, o.prev, max(opt, o.v));
    } else {
        return solve_one(ind, o.prev, opt);
    }
}

template <>
short solve_one<0>(short, const Node<0>&, short opt) {
    return opt + 1;
}

template <int D>
struct Ans {
    Node<D> o;
    short opt;
};

template <int I>
Ans<N> solve(const Node<I>& o, short opt) {
    return solve<I + 1>({solve_one(I, o, 0), o}, max(o.v, opt));
}

template <>
Ans<N> solve<N>(const Node<N>& o, short opt) {
    return {o, max(o.v, opt)};
}

template <int I>
void print_rec(const Node<I>& o, int prev, short opt) {
    if (get_item(I - 1) < prev && eq(o.v, opt)) {
        print_rec(o.prev, get_item(I - 1), opt - 1);
        print_num(get_item(I - 1));
    } else {
        print_rec(o.prev, prev, opt);
    }
}

template <>
void print_rec(const Node<0>&, int, short) {}

template <int D>
void print(const Node<D>& o, short opt) {
    print_num(opt);
    print_rec(o, 2e9, opt);
}

template <int D>
void print(const Ans<D>& ans) {
    print(ans.o, ans.opt);
}

void solve() {
    print(solve(Node<0> {}, 0));
}

int main() {
    solve();
}