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

template<typename T>
struct narr {
	int l;
	vector<int> arr;

	narr(): l(0), arr {} {}

	narr(int min, int max, T t): l(min), arr(max - min + 1, t) {}

	int &operator[](int i) {
		return arr[i - l];
	}
};

const int mbs = 5e6;
using bs = bitset<mbs>;

int k;
vector<vector<int>> ans;

void solve(vector<pair<int, int>> arr) {
	sort(begin(arr), end(arr));
	set<pair<int, int>> s(begin(arr), end(arr));
	while(sz(s)) {
		auto[xa, ta] = *s.begin();
		s.erase(s.begin());
		if(xa >= k) {
			ans.push_back({ta, k});
			if(xa - k) {
				s.emplace(xa - k, ta);
			}
		}else {
			auto[xb, tb] = *prev(s.end());
			s.erase(prev(s.end()));
			if(xa + xb >= k) {
				ans.push_back({ta, xa, tb, k - xa});
				if(xb - (k - xa)) {
					s.emplace(xb - (k - xa), tb);
				}
			}else {
				ans.clear();
				cout << -1 << endl;
				return;
			}
		}
	}
}

void solve() {
	ans.clear();
	int n, m;
	cin >> n >> m >> k;
	vector<pair<int, int>> arr(n);
	for(int i = 0; i < n; i++) {
		cin >> arr[i].first;
		arr[i].second = i + 1;
	}
	if(m >= n - 1) {
		solve(arr);
	}else {
		int mn = 0, mx = 0;
		for(auto &[a, _]: arr) {
			mn = min(mn, mn + a - k);
			mx = max(mx, mx + a - k);
		}
		bs dp[n + 1];
		assert(mx - mn + 1 <= mbs);
		dp[0][0 - mn] = true;
		for(int i = 0; i < n; i++) {
			int cur = arr[i].first - k;
			if(cur >= 0) {
				dp[i + 1] = dp[i] | (dp[i] << cur);
			}else {
				dp[i + 1] = dp[i] | (dp[i] >> (-cur));
			}
		}
		if(dp[n][-k - mn]) {
			vector<pair<int, int>> a, b;
			int x = -k;
			for(int i = n; i > 0; i--) {
				int cur = arr[i - 1].first - k;
				if(dp[i - 1][x - mn]) {
					b.push_back(arr[i - 1]);
				}else {
					assert(dp[i - 1][x - cur - mn]);
					a.push_back(arr[i - 1]);
					x -= cur;
				}
			}
			solve(a);
			solve(b);
		}else {
			cout << -1 << endl;
			return;
		}
	}
	if(sz(ans)) {
		for(auto &a: ans) {
			for(int i = 0; i < sz(a); i++) {
				if(i) {
					cout << " ";
				}
				cout << a[i];
			}
			cout << endl;
		}
	}
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}