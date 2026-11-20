#pragma GCC optimize("Ofast,unroll-loops")
#pragma GCC target("avx2")

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

const int maxn = 16e6 + 5;

int *buf, *cnt, st[1000000];
bitset<maxn> vis;

bool can_escape(int n, int m, vector<vector<int>> arr) {
	buf = new int[n * m + 5];
	const int csize = (n + 1) * (m + 1) + 5;
	cnt = new int[csize];
	memset(cnt, 0, sizeof(int) * csize);
	for (int i = 1; i <= n; i++) {
		for (int j = 1; j <= m; j++) {
			cnt[i * j]++;
		}
	}
	for (int i = 1; i < csize; i++) {
		cnt[i] += cnt[i - 1];
	}
	for (int i = 1; i <= n; i++) {
		for (int j = 1; j <= m; j++) {
			buf[--cnt[i * j]] = arr[i][j];
		}
	}
	int ptr = 0;
	auto push = [&](int u) -> void {
		if (!vis[u]) {
			vis[u] = true;
			st[ptr++] = u;
		}
	};
	push(arr[1][1]);
	while (ptr) {
		int u = st[--ptr];
		for (int i = cnt[u]; i < cnt[u + 1]; i++) {
			if (buf[i] == n * m) {
				return true;
			}
			push(buf[i]);
		}
	}
	return false;
}

void solve() {
	int n, m;
	cin >> n >> m;
	mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
	vector<vector<int>> arr(n + 1, vector<int>(m + 1));
	for (auto& a : arr) {
		for (auto& b : a) {
			b = cowng() % (n * m) + 1;
		}
	}
	if (can_escape(n, m, arr)) {
		cout << "yes" << endl;
	} else {
		cout << "no" << endl;
	}
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