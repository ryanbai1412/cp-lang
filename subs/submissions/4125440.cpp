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

const long mod = 1e9 + 7;

long bpow(long base, long exp) {
	long ans = 1;
	while (exp) {
		if (exp & 1) {
			 ans = (ans * base) % mod;
		}
		base = (base * base) % mod;
		exp >>= 1;
	}
	return ans;
}

long inv(long x) {
	return bpow(x, mod - 2);
}

int n, ans = 0;
vector<int> graph[8];

void dfs(int i) {
	if (i == n) {
		bool vis[n] {};
		int q[n];
		int qptr = 0;
		auto push = [&](int u) -> void {
			if (!vis[u]) {
				vis[u] = true;
				q[qptr++] = u;
			}
		};
		for (int j = 0; j < n; j++) {
			if (!vis[j]) {
				push(j);
				while (qptr) {
					int u = q[--qptr];
					for (auto& v : graph[u]) {
						push(v);
					}
				}
				ans++;
			}
		}
		return;
	}
	for (int j = 0; j < n; j++) {
		if (j != i) {
			graph[j].push_back(i);
			graph[i].push_back(j);
			dfs(i + 1);
			graph[i].pop_back();
			graph[j].pop_back();
		}
	}
}

void solve() {
	cin >> n;
	dfs(0);
	dbg(ans);
	cout << (ans * inv(bpow(n - 1, n))) % mod << endl;
}

int main() {
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