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

namespace s1 {

void solve(int n, int m, int k, const vector<pair<int, int>>& arr) {
	vector<int> vals[m];
	for (auto& [a, c] : arr) {
		vals[a].push_back(c);
	}
	vector<long> cur {0};
	for (auto& a : vals) {
		vector<long> nxt;
		for (auto& b : cur) {
			for (auto& c : a) {
				nxt.push_back(b + c);
				dbg(b, c);
			}
		}
		sort(begin(nxt), end(nxt));
		dbg(sz(nxt), k);
		nxt.resize(min(sz(nxt), k));
		swap(nxt, cur);
		dbg(sz(nxt), k);
	}
	cur.resize(k, -1);
	for (auto& a : cur) {
		cout << a << endl;
	}
}

}  // namespace s1

namespace s2 {

const int maxn = 4005 * 4005;

int dp[2][maxn];

void solve(int n, int m, int k, const vector<pair<int, int>>& arr) {
	vector<int> vals[m];
	for (auto& [a, c] : arr) {
		vals[a].push_back(c);
	}
	for (auto& a : vals) {
		sort(begin(a), end(a));
		if (!sz(a)) {
			for (int i = 0; i < k; i++) {
				cout << -1 << endl;
			}
			return;
		}
	}
	int cind = 0, pind = 1;
	dp[cind][0] = 1;
	for (auto& a : vals) {
		swap(cind, pind);
		memset(dp[cind], 0, sizeof(dp[cind]));
	}
}

}  // namespace s2

void solve() {
	int n, m, k;
	cin >> n >> m >> k;
	vector<pair<int, int>> arr(n);
	int mx = 0;
	for (auto& [a, b] : arr) {
		cin >> a >> b;
		a--;
		mx = max(mx, b);
	}
	for (int i = 0; i < m; i++) {
		int x, y;
		cin >> x >> y;
		assert(x == 1 && y == 1);
	}
	if (n <= 4000 && m <= 4000 && k <= 4000) {
		s1::solve(n, m, k, arr);
	} else if (n <= 4000 && m <= 4000 && mx <= 4000) {
		s2::solve(n, m, k, arr);
		return;
	}
}

int main() {
	ios_base::sync_with_stdio(false);
	cin.tie(nullptr);
	cin.exceptions(ios::failbit);
	solve();
}