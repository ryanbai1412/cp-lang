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

int GetMove();
void MakeMove(int v);

int get_move() {
    return GetMove() - 1;
}

void make_move(int v) {
    MakeMove(v + 1);
}

template <typename T>
T reversed(T t) {
    reverse(begin(t), end(t));
    return t;
}

constexpr int maxn = 2e5 + 5;

vector<int> graph[maxn], tree[maxn];
int cnt1 = 0;
bool vis1[maxn], special[maxn];
vector<vector<int>> pairings;

void dfs1(int u, int p) {
    if (!u || vis1[u]) {
        return;
    }
    if (p != -1) {
        tree[p].push_back(u);
    }
    vis1[u] = true;
    cnt1 += special[u];
    for (auto& v : graph[u]) {
        dfs1(v, u);
    }
}

vector<int> dfs2(int u) {
    vector<vector<int>> ch;
    if (special[u]) {
        ch.emplace_back();
    }
    for (auto& v : tree[u]) {
        auto cv = dfs2(v);
        if (sz(cv)) {
            ch.push_back(cv);
        }
    }
    for (int i = 0; i + 1 < sz(ch); i += 2) {
        auto cur = ch[i];
        cur.push_back(u);
        cur.insert(cur.end(), begin(ch[i + 1]), end(ch[i + 1]));
        pairings.push_back(cur);
    }
    if (sz(ch) % 2) {
        auto ans = ch.back();
        ans.push_back(u);
        return ans;
    } else {
        return {};
    }
}

void SocialEngineering(int n, int m, vector<pair<int,int>> edges) {
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
        mp[a[0]] = a;
        mp[a.back()] = reversed(a);
    }
    while (true) {
        int u = get_move();
        if (u == -1) {
            return;
        }
        for (auto& v : mp[u]) {
            if (u != v) {
                make_move(v);
            }
        }
        make_move(0);
    }
}

void solve() {

}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}