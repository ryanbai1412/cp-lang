struct Node {
    int dp_v, arr_v;
    Node *prev, *dp_trans;
};

int get_dp_v(Node* o) {
    return o ? o->dp_v : 0;
}

Node* max(Node* a, Node* b) {
    return get_dp_v(a) > get_dp_v(b) ? a : b;
}

bool eq(int a, int b) {
    return !(a < b) && !(a > b);
}

Node* solve_one(int ind, Node* m_prev, Node* o, Node* opt) {
    if (!o) {
        return new Node{get_dp_v(opt) + 1, get_item(ind), m_prev, opt};
    }

    if (o->arr_v < get_item(ind)) {
        return solve_one(ind, m_prev, o->prev, max(o, opt));
    } else {
        return solve_one(ind, m_prev, o->prev, opt);
    }
}

void print_rec(Node* o) {
    if (!o) {
        return;
    }

    print_rec(o->dp_trans);
    print_num(o->arr_v);
}

void print(Node* o) {
    print_num(o->dp_v);
    print_rec(o);
}

void solve(int n, int ind, Node* o, Node* opt) {
    if (eq(ind, n)) {
        print(max(o, opt));
        return;
    }

    solve(n, ind + 1, solve_one(ind, o, o, nullptr), max(o, opt));
}

void solve(int n) {
    solve(n, 0, nullptr, nullptr);
}

void solve() {
    solve(N);
}

int main() {
    solve();
}