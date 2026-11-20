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
	int n, m, k;
	cin >> n >> m >> k;
	set<pair<int, int>> s;
	for(int i = 0; i < n; i++) {
		int x;
		cin >> x;
		s.emplace(x, i + 1);
	}
	vector<vector<int>> ans;
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
				cout << -1 << endl;
				return;
			}
		}
	}
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