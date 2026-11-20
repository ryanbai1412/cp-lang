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

int solve(int n) {
	auto start = clock();
	if (n <= 2) {
		for (int i = 1; i <= n; i++) {
			cout << i << " \n"[i == n];
		}
		return 0;
	} else if (n == 3) {
		cout << "2 1 3" << endl;
		return 2;
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
	auto topt = [&](vector<int>& path) -> int {
		int ans = 0;
		for (int i = 0; i < n - 1; i++) {
			ans += graph[path[i]][path[i + 1]];
		}
		while (true) {
			for (int i = 0; i < n; i++) {
				for (int j = i + 2; j < n - 1; j++) {
					int a = path[i], b = path[i + 1], c = path[j], d = path[j + 1];
					int dans = graph[a][c] + graph[b][d] - graph[a][b] - graph[c][d];
					if (dans > 0) {
						ans += dans;
						reverse(path.begin() + i + 1, path.begin() + j + 1);
						goto loop;
					}
				}
			}
			break;
		loop:;
		}
		return ans;
	};
	int aopt = 0;
	vector<int> aans;
	vector<int> path(n);
	iota(begin(path), end(path), 0);
	auto movei = [&](int ind, int x) -> void {
		int cur = path[ind];
		*find(begin(path), end(path), x) = cur;
		path[ind] = x;
	};
	long time;
	if (n <= 50) {
		time = 0.1 * CLOCKS_PER_SEC;
	} else if (n <= 100) {
		time = 0.5 * CLOCKS_PER_SEC;
	} else if (n <= 500) {
		time = 1 * CLOCKS_PER_SEC;
	} else {
		time = 3 * CLOCKS_PER_SEC;
	}
	while (clock() - start < time) {
		shuffle(begin(path), end(path), cowng);
		if (n > 20) {
			movei(n - 1, n / 2 - 1);
		}
		if (n > 100) {
			movei(0, n / 2 + rint(0, 1));
		}
		int cans = topt(path);
		if (cans > aopt) {
			aopt = cans;
			aans = path;
			dbg(double(clock() - start) / CLOCKS_PER_SEC);
		}
	}
	dbg(aopt);
	for (int i = 0; i < n; i++) {
		//		cout << aans[i] + 1 << " \n"[i == n - 1];
	}
	return aopt;
}

void solve() {
	for (int i = 100; i <= 1000; i += 100) {
		for (int j = i - 10; j <= i; j++) {
			dbg(j);
			solve(j);
		}
	}
	int n;
	cin >> n;
	solve(n);
	if (n == -1) {
		vector<int> ans {0,  0,  0,  2,  7,   10,  16,  22,  30,  38, 48,
						 58, 69, 80, 95, 110, 126, 142, 160, 178, 198};
		for (int i = 1; i <= 20; i++) {
			dbg(i);
			int cans = solve(i);
			assert(cans == ans[i]);
		}
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