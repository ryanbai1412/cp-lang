#pragma GCC optimize("Ofast")

#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int IINF = 1e9;

struct NV {
    int sum, pref, suff, v;

    NV& operator+=(const NV& n) {
        v = max({v, n.v, suff + n.pref});
        pref = max(pref, sum + n.pref);
        suff = max(n.suff, suff + n.sum);
        sum += n.sum;
        return *this;
    }

    friend NV operator+(NV a, const NV& b) {
        return a += b;
    }

    static NV c_from(int x) {
        return {x, x, x, x};
    }

    static NV c_def() {
        return {0, -IINF, -IINF, -IINF};
    }
};

struct Lazy {
    static constexpr int NO_SET = INT_MIN;

    int set;
    bool rev;

    Lazy& operator+=(const Lazy& l) {
        if (l.set != NO_SET) {
            return *this = l;
        }
        rev ^= l.rev;
        return *this;
    }

    void apply_inplace(NV& n, int siz) {
        if (set != NO_SET) {
            n.sum = set * siz;
            n.pref = n.suff = n.v = max(set, set * siz);
        } else if (rev) {
            swap(n.pref, n.suff);
        }
    }

    static Lazy c_def() {
        return {NO_SET, false};
    }

    static Lazy c_set(int x) {
        return {x, false};
    }

    static Lazy c_rev() {
        return {NO_SET, true};
    }
};

struct Node {
    NV v, x;
    Lazy lazy;
    int siz;
    Node *p, *c[2];

    Node(int x)
        : v(NV::c_from(x)),
          x(v),
          lazy(Lazy::c_def()),
          siz(1),
          p(nullptr),
          c {} {}

    ~Node() {
        delete c[0];
        delete c[1];
    }

    void pushdown() {
        for (auto& a : c) {
            if (!a) {
                continue;
            }
            a->lazy += lazy;
            lazy.apply_inplace(a->v, a->siz);
        }
        lazy.apply_inplace(x, 1);
        lazy.apply_inplace(v, siz);
        if (lazy.rev) {
            swap(c[0], c[1]);
        }
        lazy = Lazy::c_def();
    }

    void maintain() {
        siz = g_siz(c[0]) + 1 + g_siz(c[1]);
        v = g_v(c[0]) + x + g_v(c[1]);
        lazy.apply_inplace(v, siz);
    }

    int p_dir() const {
        return p->c[1] == this;
    }

    void rotate() {
        //        dbg(p, this);
        auto cp = p;
        cp->pushdown();
        pushdown();
        int dir = p_dir();
        int cp_dir = cp->p ? cp->p_dir() : -1;
        cp->c[dir] = c[dir ^ 1];
        if (cp->c[dir]) {
            cp->c[dir]->p = cp;
        }
        c[dir ^ 1] = cp;
        p = cp->p;
        cp->p = this;
        cp->maintain();
        maintain();
        if (cp_dir != -1) {
            p->c[cp_dir] = this;
            p->maintain();
        }
        //        dbg(this);
    }

    void splay() {
        while (p) {
            if (!p->p) {
                rotate();
            } else if (p_dir() == p->p_dir()) {
                p->rotate();
                rotate();
            } else {
                rotate();
                rotate();
            }
        }
    }

    Node* find_kth(int ind) {
        pushdown();
        if (ind == g_siz(c[0])) {
            return this;
        }
        if (ind < g_siz(c[0])) {
            return c[0]->find_kth(ind);
        } else {
            return c[1]->find_kth(ind - g_siz(c[0]) - 1);
        }
    }

    Node* detach(int dir) {
        pushdown();
        Node* o = c[dir];
        if (!o) {
            return o;
        }
        o->p = nullptr;
        c[dir] = nullptr;
        maintain();
        return o;
    }

    static NV g_v(Node* o) {
        return o ? o->v : NV::c_def();
    }

    static int g_siz(Node* o) {
        return o ? o->siz : 0;
    }

    static pair<Node*, Node*> split(Node* o, int ind) {
        if (!o) {
            assert(!ind);
            return {nullptr, nullptr};
        }
        if (ind == o->siz) {
            return {o, nullptr};
        }
        o = o->find_kth(ind);
        o->splay();
        return {o->detach(0), o};
    }

    static tuple<Node*, Node*, Node*> split(Node* o, int ind, int len) {
        auto [l, tmp] = split(o, ind);
        auto [m, r] = split(tmp, len);
        return {l, m, r};
    }

    static Node* merge(Node* l, Node* r) {
        if (!l) {
            return r;
        } else if (!r) {
            return l;
        }
        dbg(l);
        l->pushdown();
        while (l->c[1]) {
            l = l->c[1];
            l->pushdown();
        }
        l->splay();
        dbg(l);
        l->c[1] = r;
        r->p = l;
        l->maintain();
        return l;
    }

    static Node* merge(Node* a, Node* b, Node* c) {
        return merge(merge(a, b), c);
    }
};

ostream& operator<<(ostream& out, void* ptr) {
    return out << uint64_t(ptr) % 1000;
}

ostream& operator<<(ostream& out, Node* o) {
    if (!o) {
        return out << "n";
    }
    return out << "(" << o->c[0] << ")"
               << " " << (void*)(o) << "/" << (void*)(o->p) << " "
               << "(" << o->c[1] << ")";
}

void solve() {
    int n, q;
    cin >> n >> q;
    Node* root {};
    for (int i = 0; i < n; i++) {
        int x;
        cin >> x;
        root = Node::merge(root, new Node(x));
    }
    dbg(root);
    while (q--) {
        string s;
        cin >> s;
        if (s == "MAX-SUM") {
            cout << root->v.v << endl;
            continue;
        }
        int ind, len;
        cin >> ind >> len;
        if (s == "INSERT") {
            auto [nl, nr] = Node::split(root, ind);
            for (int i = 0; i < len; i++) {
                int x;
                cin >> x;
                nl = Node::merge(nl, new Node(x));
            }
            root = Node::merge(nl, nr);
            continue;
        }
        ind--;
        auto [nl, nm, nr] = Node::split(root, ind, len);
        dbg(nl, nm, nr);
        if (s == "DELETE") {
            delete nm;
            root = Node::merge(nl, nr);
            continue;
        }
        if (s == "MAKE-SAME") {
            int x;
            cin >> x;
            nm->lazy += Lazy::c_set(x);
            nm->maintain();
        } else if (s == "REVERSE") {
            nm->lazy += Lazy::c_rev();
            nm->maintain();
        } else if (s == "GET-SUM") {
            cout << Node::g_v(nm).sum << endl;
        } else {
            assert(false);
        }
        root = Node::merge(nl, nm, nr);
        dbg(root);
    }
    delete root;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}