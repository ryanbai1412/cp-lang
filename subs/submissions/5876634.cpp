#include <iostream>
#include <vector>

using namespace std;

int N;
vector<int> arr;

int get_item(int ind) {
    return arr[ind];
}

void print_num(int x) {
    cout << x << endl;
}

bool eq(int a, int b) {
    return !(a < b) && !(a > b);
}

struct DpValue {
    int dp_v, trans_steps;
};

DpValue max(DpValue a, DpValue b) {
    return a.dp_v > b.dp_v ? a : b;
}

template <int N>
struct Node {
    int arr_v;
    DpValue v;
    Node<N - 1> prev;
};

template <>
struct Node<0> {
    DpValue v;
};

template <int O>
DpValue solve_one(int ind, const Node<O>& o, int cur_steps, DpValue opt) {
    if (o.arr_v < get_item(ind)) {
        return solve_one(ind, o.prev, cur_steps + 1,
                         max(opt, DpValue{o.v.dp_v, cur_steps}));
    } else {
        return solve_one(ind, o.prev, cur_steps + 1, opt);
    }
}

template <>
DpValue solve_one<0>(int ind, const Node<0>& o, int cur_steps, DpValue opt) {
    return {opt.dp_v + 1, opt.trans_steps};
}

template <int N>
void print_rec(const Node<N>& o, int steps) {
    if (!steps) {
        print_rec(o.prev, o.v.trans_steps - 1);
        print_num(o.arr_v);
    } else {
        print_rec(o.prev, steps - 1);
    }
}

template <>
void print_rec(const Node<0>&, int) {}

template <int N>
void print(const Node<N>& o, DpValue opt) {
    print_num(opt.dp_v);
    print_rec(o, opt.trans_steps);
}

template <int N>
DpValue solve_upd_opt(const Node<N>& o, DpValue opt) {
    return max(DpValue{o.v.dp_v, 0}, DpValue{opt.dp_v, opt.trans_steps + 1});
}

template <int N>
void solve(int n, const Node<N>& o, DpValue opt) {
    if (eq(N, n)) {
        print(o, solve_upd_opt(o, opt));
        return;
    }

    solve<N + 1>(n, {get_item(N), solve_one(N, o, 1, {0, N + 1}), o},
                 solve_upd_opt(o, opt));
}

template <>
void solve<800>(int, const Node<800>&, DpValue) {}

void solve(int n) {
    solve(n, Node<0>{}, {0, 0});
}

void solve() {
    solve(N);
}

int main() {
    cin >> N;
    arr.resize(N);
    for (auto& a : arr) {
        cin >> a;
    }
    solve();
}