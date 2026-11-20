// extern const int N;
// constexpr int get_num(int);
// void print_num(int);

#include "bits/extc++.h"

using namespace std;

struct Node {
    int dp_v, arr_v;
    Node *prev, *dp_trans;
};

int get_dp_v(Node* o) {
    return o ? o->dp_v : 0;
}

bool eq(int a, int b) {
    return !(a < b) && !(a > b);
}

Node* solve_one(int ind, Node* m_prev, Node* o, Node* opt) {
    if (!o) {
        return new Node{get_dp_v(opt) + 1, get_num(ind), m_prev, opt};
    }

    if (!(o->arr_v < get_num(ind))) {
        return solve_one(ind, m_prev, o->prev, opt);
    }
    if (get_dp_v(o) > get_dp_v(opt)) {
        return solve_one(ind, m_prev, o->prev, o);
    } else {
        return solve_one(ind, m_prev, o->prev, opt);
    }
}

void print(Node* o) {
    if (!o) {
        return;
    }

    print(o->dp_trans);
    print_num(o->arr_v);
}

void solve(int n, int ind, Node* o) {
    if (eq(ind, n)) {
        print_num(o->dp_v);
        print(o);
        return;
    }

    solve(n, ind + 1, solve_one(ind, o, o, nullptr));
}

void solve(int n) {
    solve(n, 0, nullptr);
}

int main() {
    solve(N);
}