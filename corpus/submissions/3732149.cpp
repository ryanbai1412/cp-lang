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

using ll = long long;

//imagine a language where int = long
#define long __int128

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

int besthub(int n, int _, int arr[], ll b) {
	long psum[n + 1];
	partial_sum(arr, arr + n, psum + 1);
	psum[0] = 0;
	auto sum = [&](int l, int r) -> long {
		return psum[r + 1] - psum[l];
	};
	auto cost = [&](int l, int r) -> long {
		int m = (l + r) / 2, mv = arr[m];
		int lcnt = m - l + 1, rcnt = r - m;
		return mv * lcnt - sum(l, m) + sum(m + 1, r) - mv * rcnt;
	};
	int ans = 0;
	for(int i = 0; i < n; i++) {
		int l = i, r = n - 1;
		while(l < r) {
			int m = (l + r + 1) / 2;
			if(cost(i, m) <= b) {
				l = m;
			}else {
				r = m - 1;
			}
		}
		dbg(i, l, cost(i, l));
		ans = max(ans, l - i + 1);
	}
	return ans;
}

void solve() {
	int n, m, b;
	cin >> n >> m >> b;
	auto arr = new int[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	cout << besthub(n, m, arr, b) << endl;
	delete[] arr;
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