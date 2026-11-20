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

    void rev() {
        swap(pref, suff);
    }

    static NV c_from(int x) {
        return {x, x, x, x};
    }

    static NV c_range(int len, int x) {
        int vx = max(x, len * x);
        return {vx, len * x, vx, vx};
    }

    static NV c_def() {
        return {int(-1e9), 0, int(-1e9), int(-1e9)};
    }
};

mt19937 cowng(1412);

constexpr int NO_SET = INT_MIN;

struct Node {
    NV v;
    int x, siz, pri, l_set;
    bool l_rev;
    Node *lc, *rc;

    Node(int x)
        : v(NV::c_from(x)),
          x(x),
          siz(1),
          pri(int(cowng())),
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
            c->l_set = c->x = o->l_set;
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
        o->v.rev();
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
    Node *l {}, *r {};
    split(o, ind, l, r);
    return {l, r};
}

tuple<Node*, Node*, Node*> split(Node* o, int ql, int qr) {
    auto [mm, r] = split(o, qr);
    auto [l, m] = split(mm, ql);
    return {l, m, r};
}

Node* merge(Node* l, Node* r) {
    if (!l) {
        maintain(r);
        return r;
    } else if (!r) {
        maintain(l);
        return l;
    }

    if (l->pri < r->pri) {
        pushdown(l);
        maintain(l->lc);
        l->rc = merge(l->rc, r);
        maintain(l);
        return l;
    } else {
        pushdown(r);
        maintain(r->rc);
        r->lc = merge(l, r->lc);
        maintain(r);
        return r;
    }
}

Node* merge(Node* a, Node* b, Node* c) {
    return merge(merge(a, b), c);
}

void rfree(Node* o) {
    if (!o) {
        return;
    }

    rfree(o->lc);
    rfree(o->rc);

    delete o;
}

Node* build(const vector<int>& arr, int l, int r) {
    if (l == r) {
        return nullptr;
    }

    int mid = (l + r) / 2;
    Node* o = new Node(arr[mid]);
    o->lc = build(arr, l, mid);
    o->rc = build(arr, mid + 1, r);
    maintain(o);
    return o;
}

void solve() {
    int n, q;
    cin >> n >> q;

    auto read_arr = [&](int n) -> Node* {
        vector<int> arr(n);
        for (auto& a : arr) {
            cin >> a;
        }

        return build(arr, 0, n);
    };

    Node* root = read_arr(n);

    while (q--) {
        string ty;
        cin >> ty;

        if (ty == "INSERT") {
            int ind, len;
            cin >> ind >> len;
            auto [rl, rr] = split(root, ind);
            root = merge(rl, read_arr(len), rr);
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
            rfree(rm);
            root = merge(rl, rr);
            continue;
        }

        if (ty == "MAKE-SAME") {
            int x;
            cin >> x;
            rm->l_set = x;
            maintain(rm);
        } else if (ty == "REVERSE") {
            rm->l_rev ^= true;
            maintain(rm);
        } else {
            dbg(rm->siz, len);
            cout << g_v(rm).sum << endl;
        }

        root = merge(rl, rm, rr);
    }

    rfree(root);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}