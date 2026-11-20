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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

long k;

struct Node {
    int siz;
    long pref, suff, ans;

    Node operator+(const Node& n) {
        return {siz + n.siz, max(pref, n.pref + siz * k),
                max(n.suff, suff + n.siz * k),
                max({ans, n.ans, (n.pref + suff + k) / 2})};
    }

    static Node def() {
        return {0, long(-1e18), long(-1e18), 0};
    }

    static Node from(int x) {
        return {1, -x, x, 0};
    }
};

struct ST {
    int n;
    vector<Node> v;

    ST(int n) : n(n), v(4 * n, Node::def()) {}

    void update(int o, int l, int r, int ind, Node x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = v[lc] + v[rc];
    }

    void update(int ind, Node x) {
        update(1, 0, n - 1, ind, x);
    }

    long query() const {
        return v[1].ans;
    }
};

void print(long x) {
    cout << x / 2;
    if (x & 1) {
        cout << ".5";
    }
}

void solve() {
    {
        k = 4;
        Node a = Node::from(2), b = Node::from(4), c = Node::from(6);
        dbg((a + b).ans);
    }
    int n, m;
    cin >> n >> m >> k;
    k *= 2;
    int arr[n + m];
    for (auto& a : arr) {
        cin >> a;
        a *= 2;
    }
    pair<int, int> comp[n + m];
    for (int i = 0; i < n + m; i++) {
        comp[i] = {arr[i], i};
    }
    sort(comp, comp + n + m);
    int cind[n + m];
    for (int i = 0; i < n + m; i++) {
        cind[comp[i].second] = i;
    }
    ST st(n + m);
    for (int i = 0; i < n + m; i++) {
        st.update(cind[i], Node::from(arr[i]));
        if (i >= n) {
            print(st.query());
            cout << " \n"[i == n + m - 1];
        }
    }
}

int main() {
    cout << fixed;
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}