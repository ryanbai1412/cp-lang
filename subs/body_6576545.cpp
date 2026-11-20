
inline void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}
template <typename T>
vector<T> iota(int n, T x) {
    vector<T> ans(n);
    iota(begin(ans), end(ans), x);
    return ans;
}
template <typename T>
vector<T>& operator+=(vector<T>& a, const vector<T>& b) {
    a.insert(end(a), begin(b), end(b));
    return a;
}
template <typename T>
vector<T> operator+(vector<T> a, const vector<T>& b) {
    return a += b;
}
template <typename Cb>
struct CmpByKey {
    Cb cb;
    CmpByKey(Cb cb) : cb(cb) {}
    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};
template <typename T, typename Cb>
T first_true(T l, T r, const Cb& cb) {
    while (l < r) {
        T mid = (l + r) / 2;
        if (cb(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    return l;
}
template <typename T, typename Cb>
T last_true(T l, T r, const Cb& cb) {
    while (l < r) {
        T mid = (l + r + 1) / 2;
        if (cb(mid)) {
            l = mid;
        } else {
            r = mid - 1;
        }
    }
    return l;
}
template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}
template <typename T>
T negated(T arr) {
    for (auto& a : arr) {
        a = -a;
    }
    return arr;
}
template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}
inline vector<int> inverse(const vector<int>& arr) {
    int n = sz(arr);
    vector<int> ans(n);
    for (int i = 0; i < n; i++) {
        ans[arr[i]] = i;
    }
    return ans;
}
struct NV {
    int pref, suff, sum, v;
    NV operator+(const NV& o) const {
        return {max(pref, sum + o.pref), max(suff + o.sum, o.suff), sum + o.sum,
                max({v, o.v, suff + o.pref})};
    }
    NV rev() const {
        return {suff, pref, sum, v};
    }
    static NV c_from(int x) {
        return {x, x, x, x};
    }
    static NV c_def() {
        return {int(-1e9), int(-1e9), 0, int(-1e9)};
    }
    static NV c_range(int x, int len) {
        int cx = max(x, x * len);
        return {cx, cx, x * len, cx};
    }
};
mt19937_64 cowng(1412);
constexpr int NO_SET = INT_MIN;
struct Node {
    Node *lc, *rc;
    NV v;
    int siz, pri, x, l_set;
    bool rev;
    Node(int x)
        : lc {},
          rc {},
          v(NV::c_from(x)),
          siz(1),
          pri(cowng()),
          x(x),
          l_set(NO_SET),
          rev(false) {}
};
int g_siz(Node* o) {
    return o ? o->siz : 0;
}
NV g_v(Node* o) {
    return o ? o->v : NV::c_def();
}
void maintain(Node* o) {
    if (!o) {
        return;
    }
    o->siz = g_siz(o->lc) + 1 + g_siz(o->rc);
    if (o->l_set != NO_SET) {
        o->v = NV::c_range(o->l_set, o->siz);
        return;
    }
    o->v = g_v(o->lc) + NV::c_from(o->x) + g_v(o->rc);
    if (o->rev) {
        o->v = o->v.rev();
    }
}
void pushdown(Node* o) {
    if (!o) {
        return;
    }
    if (o->l_set != NO_SET) {
        for (auto& ch : {o->lc, o->rc}) {
            if (ch) {
                ch->l_set = o->l_set;
            }
        }
        o->x = o->l_set;
        o->l_set = NO_SET;
        o->rev = false;
        return;
    }
    if (o->rev) {
        for (auto& ch : {o->lc, o->rc}) {
            if (ch) {
                ch->rev ^= true;
            }
        }
        o->rev = false;
        swap(o->lc, o->rc);
    }
}
void split(Node* o, int ind, Node*& nl, Node*& nr) {
    if (!o) {
        assert(!ind);
        nl = nr = nullptr;
        return;
    }
    pushdown(o);
    if (ind <= g_siz(o->lc)) {
        split(o->lc, ind, nl, o->lc);
        nr = o;
        maintain(o->rc);
    } else {
        split(o->rc, ind - g_siz(o->lc) - 1, o->rc, nr);
        nl = o;
        maintain(o->lc);
    }
    maintain(o);
}
pair<Node*, Node*> split(Node* o, int ind) {
    dbg(g_siz(o), ind);
    Node *nl, *nr;
    split(o, ind, nl, nr);
    return {nl, nr};
}
array<Node*, 3> split(Node* o, int ind, int len) {
    dbg(g_siz(o), ind, len);
    Node *nl, *nm, *nr;
    split(o, ind, nl, nm);
    split(nm, len, nm, nr);
    return {nl, nm, nr};
}
Node* merge(Node* a, Node* b) {
    if (!a) {
        maintain(b);
        return b;
    } else if (!b) {
        maintain(a);
        return a;
    }
    if (a->pri < b->pri) {
        pushdown(a);
        a->rc = merge(a->rc, b);
        maintain(a->lc);
        maintain(a);
        return a;
    } else {
        pushdown(b);
        b->lc = merge(a, b->lc);
        maintain(b->rc);
        maintain(b);
        return b;
    }
}
Node* merge(Node* a, Node* b, Node* c) {
    return merge(merge(a, b), c);
}
void tfree(Node* o) {
    if (!o) {
        return;
    }
    tfree(o->lc);
    tfree(o->rc);
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
    vector<int> buf;
    auto build_arr = [&](int len) -> Node* {
        buf.resize(len);
        for (auto& a : buf) {
            cin >> a;
        }
        return build(buf, 0, len);
    };
    Node* root = build_arr(n);
    while (q--) {
        string ty;
        cin >> ty;
        if (ty == "MAX-SUM") {
            cout << g_v(root).v << endl;
            continue;
        }
        int ind, len;
        cin >> ind >> len;
        if (ty == "INSERT") {
            auto [nl, nr] = split(root, ind);
            root = merge(nl, build_arr(len), nr);
            continue;
        }
        ind--;
        auto [nl, nm, nr] = split(root, ind, len);
        if (ty == "DELETE") {
            tfree(nm);
            root = merge(nl, nr);
            continue;
        }
        if (ty == "MAKE-SAME") {
            int x;
            cin >> x;
            nm->l_set = x;
            maintain(nm);
        } else if (ty == "REVERSE") {
            nm->rev ^= true;
            maintain(nm);
        } else {
            cout << g_v(nm).sum << endl;
        }
        root = merge(nl, nm, nr);
    }
    tfree(root);
}
