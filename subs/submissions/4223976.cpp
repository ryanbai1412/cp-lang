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

int rint(int l, int r) {
	return cowng() % (r - l + 1) + l;
}

void solve() {
	auto start = clock();
	int n;
	cin >> n;
	if (n <= 2) {
		for (int i = 1; i <= n; i++) {
			cout << i << " \n"[i == n];
		}
		return;
	} else if (n == 3) {
		cout << "2 1 3" << endl;
		return;
	}
	int graph[maxn][maxn] {};
	for (int i = 0; i < n; i++) {
		for (int j = 0; j < n; j++) {
			int w = abs(i - j);
			if (prime[w]) {
				graph[i][j] = w;
			}
		}
	}
	auto tans = [&](const vector<int>& arr) -> pair<int, vector<int>> {
		int ans = 0;
		for (int i = 0; i < n - 1; i++) {
			ans += graph[arr[i]][arr[i + 1]];
		}
		return {ans, arr};
	};
	auto topt = [&](vector<int> path) -> vector<int> {
		int nxt[n];
		for (int i = 0; i < n - 1; i++) {
			nxt[path[i]] = path[i + 1];
		}
		nxt[path.back()] = -1;
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
	auto climb = [&](vector<int> path, int iters) -> pair<int, vector<int>> {
		auto ans = tans(topt(path));
		for (int i = 0; i < iters; i++) {
			int a = cowng() % n;
			int b;
			while ((b = cowng() % n) == a) ;
			swap(path[a], path[b]);
			auto cans = tans(topt(path));
			if (cans > ans) {
				ans = cans;
			} else {
				swap(path[a], path[b]);
			}
		}
		return ans;
	};
	pair<int, vector<int>> ans = tans(topt(greedy(n / 2 - 1)));
	vector<int> path(n);
	iota(begin(path), end(path), 0);
	auto movei = [&](int ind, int x) -> void {
		int cur = path[ind];
		*find(begin(path), end(path), x) = cur;
		path[ind] = x;
	};
	long time = 0;
	if (n <= 50) {
		time = 0.1 * CLOCKS_PER_SEC;
	} else if (n <= 100) {
		time = 0.5 * CLOCKS_PER_SEC;
	} else {
		time = 4.9 * CLOCKS_PER_SEC;
	}
	while (clock() - start < time) {
		shuffle(begin(path), end(path), cowng);
		movei(n - 1, n / 2 - 1);
		if (n > 100) {
			movei(0, n / 2 + rint(1, 2));
		}
		ans = max(ans, tans(topt(path)));
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