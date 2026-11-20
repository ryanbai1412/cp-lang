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
#define dbg(...) 1412
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

const int maxn = 1e3 + 5;

void solve() {
	auto start = clock();
	int n, m, ia, ib;
	cin >> n >> m >> ia >> ib;
	ia--;
	ib--;
	vector<int> graph[n];
	for (int i = 0; i < m; i++) {
		int u, v;
		cin >> u >> v;
		u--;
		v--;
		graph[u].push_back(v);
		graph[v].push_back(u);
	}

	int nxt[n][n];
	for (int i = 0; i < n; i++) {
		bool vis[n] {};
		vis[i] = true;
		vector<int> q, nq;
		nq.push_back(i);
		while (sz(nq)) {
			swap(q, nq);
			sort(begin(q), end(q));
			nq.clear();
			for (auto& u : q) {
				for (auto& v : graph[u]) {
					if (!vis[v]) {
						vis[v] = true;
						nxt[v][i] = u;
						nq.push_back(v);
					}
				}
			}
		}
		nxt[i][i] = i;
	}
	mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
	long ans = 0, iters = 0;
	while ((clock() - start) < 0.59 * CLOCKS_PER_SEC) {
		iters++;
		int a = ia, b = ib, i;
		for (i = 1; ; i++) {
			a = nxt[nxt[a][b]][b];
			if (a == b) {
				break;
			}
			int csz = sz(graph[b]);
			int cnxt = cowng() % (csz + 1);
			if (cnxt < csz) {
				b = graph[b][cnxt];
			}
			if (a == b) {
				break;
			}
		}
		ans += i;
	}
	cout << double(ans) / iters << endl;
}

int main() {
	cout << fixed << setprecision(3);
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}