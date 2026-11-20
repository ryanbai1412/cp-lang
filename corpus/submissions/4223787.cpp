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

const int maxn = 1005;

mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
bool prime[maxn + 1];
vector<int> primes;

void pcomp() {
	memset(prime, 1, sizeof(prime));
	prime[0] = prime[1] = false;
	for (int i = 2; i <= maxn; i++) {
		if (prime[i]) {
			primes.push_back(i);
			if (long(i) * i <= maxn) {
				for (int j = i * i; j <= maxn; j += i) {
					prime[j] = false;
				}
			}
		}
	}
}

void solve() {
	int n;
	cin >> n;
	int graph[maxn][maxn] {};
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			int w = abs(i - j);
			if (prime[w]) {
				graph[i][j] = w;
			}
		}
	}
	auto eval = [&](const vector<int>& arr) -> int {
		int ans = 0;
		for (int i = 0; i < n - 1; i++) {
			ans += graph[arr[i]][arr[i + 1]];
		}
		return ans;
	};
	auto topt = [&](vector<int> path) -> vector<int> {
		while (true) {
			for (int i = 0; i < n; i++) {
				for (int j = i + 2; j < n - 1; j++) {
					int a = path[i], b = path[i + 1], c = path[j], d = path[j + 1];
					if (graph[a][c] + graph[b][d] > graph[a][b] + graph[c][d]) {
						reverse(path.begin() + i + 1, path.begin() + j + 1);
						goto loop;
					}
				}
			}
			break;
		loop:;
		}
		return path;
	};
	auto greedy = [&](int start) -> vector<int> {
		vector<int> path;
		bool vis[n] {};
		auto push = [&](int u) -> void {
			path.push_back(u);
			vis[u] = true;
		};
		push(start);
		while (sz(path) < n) {
			int u = path.back();
			pair<int, int> opt {0, 0};
			for (int i = 0; i < n; i++) {
				if (!vis[i]) {
					opt = max(opt, pair<int, int> {graph[u][i], i});
				}
			}
			push(opt.second);
		}
		return path;
	};
	pair<int, vector<int>> ans {0, {}};
	auto upd = [&](const vector<int>& x) -> void {
		ans = max(ans, pair<int, vector<int>> {eval(x), x});
	};
	for (int i = 0; i < n; i++) {
		upd(topt(greedy(i)));
	}
	vector<int> path(n);
	iota(begin(path), end(path), 0);
	auto start = clock();
	while (clock() - start < long(1.9 * CLOCKS_PER_SEC)) {
		shuffle(begin(path), end(path), cowng);
		upd(topt(path));
	}
	dbg(ans.first);
	for (int i = 0; i < n; i++) {
		cout << ans.second[i] + 1 << " \n"[i == n - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	int t = 1;
	// cin >> t;
	for (int _ = 1; _ <= t; _++) {
		dbg(_);
		// cout << "Case #" << _ << ": ";
		solve();
	}
}