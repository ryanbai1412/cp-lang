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

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

struct Grader {
    int n, m, cu = 1;
    mt19937 cowng;
    set<pair<int, int>> edges;

    Grader() {}
    Grader(int n, int m, const vector<pair<int, int>>& _edges) : n(n), m(m) {
        for (auto& [u, v] : _edges) {
            edges.emplace(u, v);
            edges.emplace(v, u);
        }
        int seed;
        cin >> seed;
        cowng = mt19937(seed);
    }

    void m_make_move(int v) {
        assert(edges.count({cu, v}));
        edges.erase({cu, v});
        edges.erase({v, cu});
        cu = v;
    }

    int get_move() {
        assert(cu == 1);
        vector<int> adj;
        for (auto& [u, v] : edges) {
            if (u == 1) {
                adj.push_back(v);
            }
        }
        if (!sz(adj)) {
            return 0;
        }
        int u = adj[cowng() % sz(adj)];
        m_make_move(u);
        return u;
    }

    void make_move(int v) {
        m_make_move(v);
    }
} grader;

#ifdef LOCAL
int GetMove() {
    return grader.get_move();
}
void MakeMove(int v) {
    grader.make_move(v);
}
#else
int GetMove();
void MakeMove(int v);
#endif

int get_move() {
    int ans = GetMove() - 1;
    dbg("G", ans);
    return ans;
}

void make_move(int v) {
    dbg("M", v);
    MakeMove(v + 1);
}

template <typename T>
T reversed(T t) {
    reverse(begin(t), end(t));
    return t;
}

constexpr int maxn = 2e5 + 5;

vector<int> graph[maxn], tree[maxn];
int cnt1 = 0, par[maxn];
bool vis1[maxn], special[maxn];
vector<vector<int>> pairings;

void dfs1(int u, int p) {
    if (!u || vis1[u]) {
        return;
    }
    par[u] = p;
    if (p != -1) {
        tree[p].push_back(u);
    }
    vis1[u] = true;
    cnt1 += special[u];
    for (auto& v : graph[u]) {
        dfs1(v, u);
    }
}

int dfs2(int u) {
    vector<int> ch;
    if (special[u]) {
        ch.push_back(u);
    }
    for (auto& v : tree[u]) {
        auto cv = dfs2(v);
        if (cv != -1) {
            ch.push_back(cv);
        }
    }
    for (int i = 0; i + 1 < sz(ch); i += 2) {
        vector<int> cur;
        auto go = [&](int v) -> void {
            while (u != v) {
                cur.push_back(v);
                v = par[v];
            }
        };
        go(ch[i]);
        auto tmp = reversed(cur);
        cur.clear();
        go(ch[i + 1]);
        cur.push_back(u);
        cur.insert(cur.end(), begin(tmp), end(tmp));
        pairings.push_back(cur);
    }
    if (sz(ch) % 2) {
        return ch.back();
    } else {
        return -1;
    }
}

void SocialEngineering(int, int, vector<pair<int, int>> edges) {
    for (auto& [u, v] : edges) {
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    if (sz(graph[0]) % 2) {
        return;
    }
    for (auto& u : graph[0]) {
        special[u] = true;
    }
    vector<int> roots;
    for (auto& u : graph[0]) {
        if (vis1[u]) {
            continue;
        }
        roots.push_back(u);
        dfs1(u, -1);
        if (cnt1 % 2) {
            return;
        }
    }
    for (auto& a : roots) {
        dfs2(a);
    }
    map<int, vector<int>> mp;
    for (auto& a : pairings) {
        dbg(pairings);
        mp[a[0]] = a;
        mp[a.back()] = reversed(a);
    }
    while (true) {
        int u = get_move();
        if (u == -1) {
            return;
        }
        auto& cans = mp[u];
        for (auto& v : cans) {
            if (u != v) {
                make_move(v);
            }
        }
        make_move(0);
    }
}

void solve() {
    int n, m;
    cin >> n >> m;
    vector<pair<int, int>> edges(m);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
    }
    grader = Grader(n, m, edges);
    SocialEngineering(n, m, edges);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}