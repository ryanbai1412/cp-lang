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
#define sz(x) int((x).size())

const int maxn = 3e3 + 5;

int dist[maxn][maxn];

void solve() {
	int n, m;
	cin >> n >> m;
	vector<int> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}
	memset(dist, -1, sizeof(dist));
	for (int i = 0; i < n; i++) {
		queue<int> q;
		auto push = [&](int u, int d) -> void {
			if (dist[i][u] == -1) {
				dist[i][u] = d;
				q.push(u);
			}
		};
		push(i, 0);
		while (sz(q)) {
			int u = q.front();
			q.pop();
			for (auto& v : graph[u]) {
				push(v, dist[i][u] + 1);
			}
		}
	}
	int q;
	cin >> q;
	while (q--) {
		int a, b, c, d, e, f;
		cin >> a >> b >> c >> d >> e >> f;
		a--;
		b--;
		d--;
		e--;
		if (dist[a][b] <= c || dist[d][e] <= f) {
			cout << "YES" << endl;
		} else {
			c--;
			f--;
			dbg(dist[a][d], dist[b][e], c, f);
			if (min(dist[a][d] + dist[b][e], dist[a][e] + dist[b][d]) <= c + f) {
				cout << "YES" << endl;
			} else {
				cout << "NO" << endl;
			}
		}
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	solve();
}