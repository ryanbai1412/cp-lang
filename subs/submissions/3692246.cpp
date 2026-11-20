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

#define int long

void solve() {
	int n, cl, cr;
	cin >> n >> cl >> cr;
	int arr[n + 1];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	arr[n] = 0;
	queue<pair<int, int>> q;
	long ans = 0;
	int prev = 0;
	for(int i = 0; i <= n; i++) {
		if(prev <= arr[i]) {
			q.emplace(i, arr[i] - prev);
			ans += arr[i] - prev;
		}else {
			int sub = prev - arr[i];
			while(sub > 0) {
				if(q.empty()) {
					cout << -1 << endl;
					return;
				}
				auto[ind, x] = q.front();
				q.pop();
				int csub = min(sub, x);
				sub -= csub;
				x -= csub;
				int dist = i - ind;
				if(dist < cl) {
					cout << -1 << endl;
					return;
				}else if(cr < dist) {
					ans += csub;
				}
				if(x) {
					q.emplace(ind, x);
				}
			}
		}
		prev = arr[i];
	}
	cout << ans << endl;
}

int32_t main() {
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