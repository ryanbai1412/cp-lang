#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

struct DS {
	int ind, k;
	deque<pair<int, int>> q;

	DS(int k): ind(0), k(k), q() {}

	int add(int x) {
		ind++;
		while(sz(q) && q.back().first + ind <= x) {
			q.pop_back();
		}
		q.emplace_back(x - ind, ind);
		if(q.front().second < ind - k) {
			q.pop_front();
		}
		return q.front().first + ind;
	}

	void clear() {
		q.clear();
	}
};

void solve() {
	int n, m, x, y, q;
	cin >> n >> m >> x >> y >> q;
	bool arr[n][m];
	int dp[2][n][m];
	for(int i = 0; i < n; i++) {
		string s;
		cin >> s;
		for(int j = 0; j < m; j++) {
			arr[i][j] = s[j] == 'x';
		}
	}
	for(int i = 0; i < 2; i++) {
		for(int j = 0; j < n; j++) {
			fill(dp[i][j], dp[i][j] + m, -1e9);
		}
	}
	int cind = 0, pind = 1;
	dp[cind][x - 1][y - 1] = 0;
	for(int _ = 0; _ < q; _++) {
		swap(cind, pind);
		int tl, tr, d;
		cin >> tl >> tr >> d;
		int t = tr - tl + 1;
		DS ds(t);
		if(d == 1) {
			for(int i = 0; i < m; i++) {
				ds.clear();
				for(int j = n - 1; j >= 0; j--) {
					if(arr[j][i]) {
						ds.clear();
					}else {
						dp[cind][j][i] = ds.add(dp[pind][j][i]);
					}
				}
			}
		}else if(d == 2) {
			for(int i = 0; i < m; i++) {
				ds.clear();
				for(int j = 0; j < n; j++) {
					if(arr[j][i]) {
						ds.clear();
					}else {
						dp[cind][j][i] = ds.add(dp[pind][j][i]);
					}
				}
			}
		}else if(d == 3) {
			for(int i = 0; i < n; i++) {
				ds.clear();
				for(int j = m - 1; j >= 0; j--) {
					if(arr[i][j]) {
						ds.clear();
					}else {
						dp[cind][i][j] = ds.add(dp[pind][i][j]);
					}
				}
			}
		}else {
			for(int i = 0; i < n; i++) {
				ds.clear();
				for(int j = 0; j < m; j++) {
					if(arr[i][j]) {
						ds.clear();
					}else {
						dp[cind][i][j] = ds.add(dp[pind][i][j]);
					}
				}
			}
		}
		dbg(dp[cind][3][2], dp[cind][1][2], dp[cind][1][0]);
	}
	int ans = 0;
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < m; j++) {
			ans = max(ans, dp[cind][i][j]);
		}
	}
	cout << ans << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}