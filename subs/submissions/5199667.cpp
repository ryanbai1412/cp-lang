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

constexpr int maxn = 2e5 + 5, logn = 20;

struct TDS {
    vector<int> graph[maxn];
    int lift[logn][maxn], depth[maxn], tin[maxn], tout[maxn], tt = 0;

    TDS(int n, const vector<pair<int, int>>& edges) {
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
        pdfs(0, -1, 0);
        for (int i = 1; i < logn; i++) {
            for (int j = 0; j < n; j++) {
                if (lift[i - 1][j] == -1) {
                    lift[i][j] = -1;
                } else {
                    lift[i][j] = lift[i - 1][lift[i - 1][j]];
                }
            }
        }
    }

    void pdfs(int u, int p, int d) {
        tin[u] = tt++;
        lift[0][u] = p;
        depth[u] = d;
        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }
            pdfs(v, u, d + 1);
        }
        tout[u] = tt - 1;
    }

    bool anc(int u, int v) const {
        return tin[u] <= tin[v] && tout[v] <= tout[u];
    }

    int lca(int u, int v) const {
        if (anc(u, v)) {
            return u;
        }
        for (int i = logn - 1; i >= 0; i--) {
            int cu = lift[i][u];
            if (cu != -1 && !anc(cu, v)) {
                u = cu;
            }
        }
        return lift[0][u];
    }

    int dist(int u, int v) const {
        int l = lca(u, v);
        return depth[u] + depth[v] - 2 * depth[l];
    }
};

struct DS1 {
    struct Seg {
        mutable int l, r;

        bool operator<(const Seg& s) const {
            return l < s.l;
        }
    };

    map<Seg, int> mp;

    DS1(const vector<int>& arr) {
        for (int i = 0; i < sz(arr); i++) {
            mp[{i, i}] = arr[i];
        }
    }

    template <typename Cb>
    void update(int l, int r, int x, const Cb& cb) {
        auto it = mp.lower_bound({r + 1, -1});
        while (it != mp.begin()) {
            --it;
            auto [cl, cr] = it->first;
            int cx = it->second;
            if (cl < l) {
                if (r < cr) {
                    cb(l, r, cx);
                    it->first.r = l - 1;
                    mp[{l, r}] = x;
                    mp[{r + 1, cr}] = cx;
                    return;
                } else if (cr < l) {
                    break;
                }
                cb(l, cr, cx);
                it->first.r = l - 1;
                break;
            } else if (r < cr) {
                cb(cl, r, cx);
                it->first.l = r + 1;
            } else {
                cb(cl, cr, cx);
                it = mp.erase(it);
            }
        }
        mp[{l, r}] = x;
    }

    int query_loc(int ind) const {
        auto it = mp.lower_bound({ind + 1, -1});
        return (--it)->second;
    }
};

struct NV {
    long mx;

    NV() : mx(-1e18) {}
    NV(long x) : mx(x) {}

    NV& operator+=(const NV& n) {
        mx = max(mx, n.mx);
        return *this;
    }

    friend NV operator+(NV a, const NV& b) {
        return a += b;
    }
};

struct Lazy {
    long add;

    Lazy() : add(0) {}
    Lazy(long x) : add(x) {}

    Lazy& operator+=(const Lazy& l) {
        add += l.add;
        return *this;
    }

    void apply_inplace(NV& n) const {
        n.mx += add;
    }
};

//mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
mt19937 cowng(1412);

struct Node {
    NV v, x;
    Lazy lazy;
    int ind, pri;
    Node *lc, *rc;

    Node(int i, int x)
        : v(x), x(v), lazy(), ind(i), pri(int(cowng())), lc(nullptr), rc(nullptr) {}

    ~Node() {
        delete lc;
        delete rc;
    }
};

NV get_v(Node* o) {
    return o ? o->v : NV();
}

void pushdown(Node* o) {
    auto go = [&](Node* c) -> void {
        if (!c) {
            return;
        }
        c->lazy += o->lazy;
        o->lazy.apply_inplace(c->v);
    };
    go(o->lc);
    go(o->rc);
    o->lazy.apply_inplace(o->x);
    o->lazy = Lazy();
}

void maintain(Node* o) {
    o->v = get_v(o->lc) + o->x + get_v(o->rc);
    o->lazy.apply_inplace(o->v);
}

void split(Node* o, int ind, Node*& l, Node*& r) {
    if (!o) {
        l = r = nullptr;
        return;
    }
    pushdown(o);
    if (ind <= o->ind) {
        split(o->lc, ind, l, o->lc);
        r = o;
    } else {
        split(o->rc, ind, o->rc, r);
        l = o;
    }
    maintain(o);
}

Node* merge(Node* l, Node* r) {
    if (!l) {
        return r;
    } else if (!r) {
        return l;
    }
    if (l->pri < r->pri) {
        pushdown(l);
        l->rc = merge(l->rc, r);
        maintain(l);
        return l;
    } else {
        pushdown(r);
        r->lc = merge(l, r->lc);
        maintain(r);
        return r;
    }
}

pair<Node*, Node*> split(Node* o, int m) {
    Node *nl, *nr;
    split(o, m, nl, nr);
    return {nl, nr};
}

tuple<Node*, Node*, Node*> split(Node* o, int l, int r) {
    Node *nl, *nm, *nr;
    split(o, l, nl, nr);
    split(nr, r + 1, nm, nr);
    return {nl, nm, nr};
}

Node* merge(Node* nl, Node* nm, Node* nr) {
    return merge(merge(nl, nm), nr);
}

struct DS2 {
    vector<Node*> arr;

    DS2(int n, const vector<int>& _arr) : arr(n) {
        for (int i = 0; i < sz(_arr); i++) {
            int a = _arr[i];
            arr[a] = merge(arr[a], new Node(i, 0));
        }
    }

    ~DS2() {
        for (auto& a : arr) {
            delete a;
        }
    }

    void update_move_add(int l, int r, int qx, int nx, long add) {
        dbg(l, r, qx, nx, add);
        Node* cn;
        {
            auto [nl, nm, nr] = split(arr[qx], l, r);
            arr[qx] = merge(nl, nr);
            nm->lazy += add;
            maintain(nm);
            cn = nm;
        }
        auto [nl, nr] = split(arr[nx], l);
        dbg(cn->v.mx);
        arr[nx] = merge(nl, cn, nr);
        dbg(arr[nx]->v.mx);
    }

    void update_add(int qx, long add) {
        if (!arr[qx]) {
            return;
        }
        arr[qx]->lazy += add;
        maintain(arr[qx]);
    }

    long query(int ind, int qx) {
        dbg(ind, qx, arr[qx]->v.mx);
        auto [nl, nm, nr] = split(arr[qx], ind, ind);
        dbg(nm->v.mx);
        long ans = nm->v.mx;
        arr[qx] = merge(nl, nm, nr);
        return ans;
    }
};

void solve() {
    int n, m, q;
    cin >> n >> m >> q;
    vector<int> arr(m);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    vector<pair<int, int>> edges(n - 1);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
    }
    DS1 ds1(arr);
    DS2 ds2(n, arr);
    TDS tds(n, edges);
    while (q--) {
        char c;
        cin >> c;
        if (c == 't') {
            int l, r, x;
            cin >> l >> r >> x;
            l--;
            r--;
            x--;
            ds1.update(l, r, x, [&](int pl, int pr, int px) -> void {
                ds2.update_move_add(pl, pr, px, x, -tds.dist(px, x));
            });
        } else if (c == 'e') {
            int ind, x;
            cin >> ind >> x;
            ind--;
            ds2.update_add(ind, x);
        } else {
            int ind;
            cin >> ind;
            ind--;
            cout << ds2.query(ind, ds1.query_loc(ind)) << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}