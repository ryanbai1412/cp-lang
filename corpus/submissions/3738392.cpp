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

void solve() {
	int q, k, left = 0, ind = 0, added = 0;
	long lazy = 0;
	cin >> q >> k;
	rt<pair<int, int>> ds;
	while(q--) {
		char c;
		long x;
		cin >> c >> x;
		if(c == 'I') {
			added++;
			ds.insert({x - lazy, ind++});
		}else if(c == 'A') {
			lazy += x;
		}else if(c == 'S') {
			lazy -= x;
		}else {
			if(x > sz(ds)) {
				cout << -1 << endl;
			}else {
				cout << lazy + ds.find_by_order(sz(ds) - x)->first << endl;
			}
		}
		while(sz(ds) && lazy + ds.begin()->first < k) {
			ds.erase(ds.begin());
			left++;
		}
	}
	assert(left == added - sz(ds));
	cout << left << endl;
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