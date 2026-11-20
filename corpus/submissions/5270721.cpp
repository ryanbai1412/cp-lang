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

struct DS2 {
    struct Node {
        Node *lc, *rc;
        int x;

        Node(int x) : lc {}, rc {}, x(x) {}

        void drop() {
            if (lc) {
                lc->drop();
            }
            if (rc) {
                rc->drop();
            }
            delete lc;
            delete rc;
        }

        static Node* merge(Node* a, Node* b) {
            if (!a) {
                return b;
            } else if (!b) {
                return a;
            }
            if (a->x < b->x) {
                swap(a, b);
            }
            a->rc = merge(a->rc, b);
            swap(a->lc, a->rc);
            return a;
        }
    };

    int siz;
    Node* root;

    DS2(int x) : siz(1), root(new Node(x)) {}

    void drop() {
        root->drop();
        delete root;
    }

    void u_insert(int x) {
        siz++;
        root = Node::merge(root, new Node(x));
    }

    void u_pop_max() {
        siz--;
        delete exchange(root, Node::merge(root->lc, root->rc));
    }

    int q_max() {
        return root->x;
    }

    int size() const {
        return siz;
    }

    int q_max2() {
        int x = q_max();
        u_pop_max();
        int ans = q_max();
        u_insert(x);
        return ans;
    }

    static DS2 merge(DS2 a, const DS2& b) {
        a.root = Node::merge(a.root, b.root);
        a.siz += b.siz;
        return a;
    }
};

struct DS1 {
    int n;
    DS2 ds;

    DS1(int x) : n(1), ds {x} {}
    DS1(int n, const DS2& _ds) : n(n), ds(_ds) {
        while (sz(ds) > (n + 1) / 2 + 1) {
            ds.u_pop_max();
        }
    }

    void drop() {
        ds.drop();
    }

    int query() {
        int x = sz(ds) - n / 2;
        if (x == 1) {
            return ds.q_max();
        } else if (x == 2) {
            return ds.q_max2();
        } else {
            assert(false);
        }
    }

    static DS1 merge(DS1 a, const DS1& b) {
        return {a.n + b.n, DS2::merge(a.ds, b.ds)};
    }
};

struct S1 {
    int l, r;
    DS1 ds;

    void drop() {
        ds.drop();
    }

    int query() {
        return ds.query();
    }

    static S1 merge(S1 a, const S1& b) {
        assert(a.r == b.l);
        return {a.l, b.r, DS1::merge(a.ds, b.ds)};
    }
};

void solve() {
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    for (int i = 0; i < n; i++) {
        arr[i] -= i;
    }
    vector<S1> vals;
    for (int i = 0; i < n; i++) {
        S1 cs {i, i + 1, arr[i]};
        while (sz(vals) && vals.back().query() > cs.query()) {
            cs = S1::merge(vals.back(), cs);
            vals.pop_back();
        }
        vals.push_back(cs);
    }
    int ans = 0;
    for (int i = 0; i < sz(vals); i++) {
        auto& [l, r, ds] = vals[i];
        int mx = ds.query();
        for (int j = l; j < r; j++) {
            ans += abs(arr[j] - mx);
        }
    }
    for (auto& a : vals) {
        a.drop();
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}