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

const int maxn = 1e6 + 5;

vector<int> graph[maxn];
int siz[maxn];
pair<int, int> ans;

void pdfs(int u, int p = -1) {
    for (auto& v : graph[u]) {
        if (v != p) {
            pdfs(v, u);
            siz[u] += siz[v];
        }
    }
}

void dfs(int u, int p = -1) {
    int cans = 0;
    for (auto& a : graph[u]) {
        cans = max(cans, siz[a]);
    }
    ans = min(ans, pair<int, int> {cans, u});
    for (auto& v : graph[u]) {
        if (v != p) {
            siz[u] -= siz[v];
            siz[v] += siz[u];
            dfs(v, u);
            siz[v] -= siz[u];
            siz[u] += siz[v];
        }
    }
}

int LocateCentre(int n, int arr[], int u[], int v[]) {
    for (int i = 0; i < n - 1; i++) {
        graph[u[i]].push_back(v[i]);
        graph[v[i]].push_back(u[i]);
    }
    copy(arr, arr + n, siz);
    pdfs(0);
    ans = {INT_MAX, -1};
    dfs(0);
    return ans.second;
}
// Sample grader: n, the fans of each city, then the n - 1 roads; prints the centre.
int main() {
    int n;
    scanf("%d", &n);
    vector<int> p(n), u(max(n - 1, 1)), v(max(n - 1, 1));
    for (auto& x : p) scanf("%d", &x);
    for (int i = 0; i + 1 < n; i++) scanf("%d %d", &u[i], &v[i]);
    printf("%d\n", LocateCentre(n, p.data(), u.data(), v.data()));
}
