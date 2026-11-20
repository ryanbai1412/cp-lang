#pragma GCC optimize("Ofast")

#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct NV {
    int v, sum, pref, suff;

    NV operator+(const NV& n) const {
        return {max({v, n.v, suff + n.pref}), sum + n.sum,
                max(pref, sum + n.pref), max(suff + n.sum, n.suff)};
    }

    NV rev() const {
        return {v, sum, suff, pref};
    }

    static NV c_from(int x) {
        return {x, x, x, x};
    }

    static NV c_range(int len, int x) {
        return {max(x, len * x), len * x, max(x, len * x), max(x, len * x)};
    }

    static NV c_def() {
        return {int(-1e9), 0, int(-1e9), int(-1e9)};
    }
};

mt19937 cowng(1412);

constexpr int NO_SET = -1e9;

struct Node {
    NV v;
    int x, pri, siz, l_set;
    bool l_rev;
    Node *lc, *rc;

    Node(int x)
        : v(NV::c_from(x)),
          x(x),
          pri(int(cowng())),
          siz(1),
          l_set(NO_SET),
          l_rev(false),
          lc {},
          rc {} {}
};

int siz(Node* o) {
    return o ? o->siz : 0;
}

NV g_v(Node* o) {
    return o ? o->v : NV::c_def();
}

void pushdown(Node* o) {
    for (auto& c : {o->lc, o->rc}) {
        if (!c) {
            continue;
        }

        c->l_rev ^= o->l_rev;
        if (o->l_set != NO_SET) {
            c->x = c->l_set = o->l_set;
        }
    }

    if (o->l_rev) {
        swap(o->lc, o->rc);
        o->l_rev = false;
    }

    if (o->l_set != NO_SET) {
        o->x = o->l_set;
        o->l_set = NO_SET;
    }
}

void maintain(Node* o) {
    if (!o) {
        return;
    }

    o->siz = siz(o->lc) + 1 + siz(o->rc);

    if (o->l_set != NO_SET) {
        o->v = NV::c_range(o->siz, o->l_set);
    } else {
        o->v = g_v(o->lc) + NV::c_from(o->x) + g_v(o->rc);
    }

    if (o->l_rev) {
        o->v = o->v.rev();
    }
}

void split(Node* o, int ind, Node*& l, Node*& r) {
    if (!o) {
        l = r = nullptr;
        return;
    }

    pushdown(o);

    if (ind <= siz(o->lc)) {
        split(o->lc, ind, l, o->lc);
        maintain(o->rc);
        r = o;
    } else {
        split(o->rc, ind - siz(o->lc) - 1, o->rc, r);
        maintain(o->lc);
        l = o;
    }

    maintain(o);
}

pair<Node*, Node*> split(Node* o, int ind) {
    Node *out_l {}, *out_r {};
    split(o, ind, out_l, out_r);
    return {out_l, out_r};
}

tuple<Node*, Node*, Node*> split(Node* o, int ql, int qr) {
    auto [mid, out_r] = split(o, qr);
    auto [out_l, out_m] = split(mid, ql);
    return {out_l, out_m, out_r};
}

Node* merge(Node* l, Node* r) {
    if (!l) {
        maintain(r);
        return r;
    } else if (!r) {
        maintain(l);
        return l;
    }

    int n_sum = l->v.sum + r->v.sum;
    if (l->pri < r->pri) {
        pushdown(l);
        maintain(l->lc);
        l->rc = merge(l->rc, r);
        maintain(l);
        dbg(n_sum, l->v.sum);
        return l;
    } else {
        int c_set = r->l_set;
        pushdown(r);
        maintain(r->rc);
        r->lc = merge(l, r->lc);
        maintain(r);
        dbg(n_sum, r->v.sum, c_set);
        return r;
    }
}

Node* merge(Node* o1, Node* o2, Node* o3) {
    return merge(merge(o1, o2), o3);
}

void r_free(Node* o) {
    if (!o) {
        return;
    }

    r_free(o->lc);
    r_free(o->rc);

    delete o;
}

void solve() {
    int n, q;
    cin >> n >> q;

    auto read_arr = [&](int n) -> Node* {
        Node* o {};

        for (int i = 0; i < n; i++) {
            int x;
            cin >> x;
            o = merge(o, new Node(x));
        }

        return o;
    };

    Node* root = read_arr(n);

    while (q--) {
        string ty;
        cin >> ty;

        if (ty == "INSERT") {
            int ind, len;
            cin >> ind >> len;

            auto o = read_arr(len);

            auto [rl, rr] = split(root, ind);
            root = merge(rl, o, rr);
            continue;
        } else if (ty == "MAX-SUM") {
            cout << g_v(root).v << endl;
            continue;
        }

        int ind, len;
        cin >> ind >> len;
        ind--;

        auto [rl, rm, rr] = split(root, ind, ind + len);

        if (ty == "DELETE") {
            r_free(rm);
            root = merge(rl, rr);
            continue;
        }

        if (ty == "MAKE-SAME") {
            int x;
            cin >> x;
            rm->l_set = x;
            maintain(rm);
            dbg(rl, rr, rm->v.sum);
        } else if (ty == "REVERSE") {
            rm->l_rev ^= true;
            maintain(rm);
        } else if (ty == "GET-SUM") {
            cout << g_v(rm).sum << endl;
        }

        if (rl && rm && rr) {
            dbg(rl->v.sum, rm->v.sum, rr->v.sum);
        }
        root = merge(rl, rm, rr);
        dbg(root->v.sum);
    }

    r_free(root);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}