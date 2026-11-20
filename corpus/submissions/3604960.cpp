#include "bits/extc++.h"

using namespace std;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T, class U>
void sep(T &out, const string &s, const U &u) {
	out << u;
}

template<class T, class Head, class ...Tail>
void sep(T &out, const string &s, const Head &h, const Tail &...t) {
	out << h << s;
	sep(out, s, t...);
}

#ifdef DEBUG
#define dbg(...)                                                      \
cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": ";       \
sep(cerr, " | ", __VA_ARGS__);                                        \
cerr << endl
#else
#define cerr if(false) cerr
#define dbg(...) cerr
#endif

using ld = long double;

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

const ld pi = M_PI;

void solve() {
	int n;
	cin >> n;
	array<int, 3> arr[n];
	for(int i = 0; i < n; i++) {
		for(int j = 0; j < 3; j++) {
			cin >> arr[i][j];
		}
	}
	int ans = 0;
	for(int i = 0; i < n; i++) {
		vector<pair<ld, int>> cur;
		cur.reserve(n);
		auto &[cx, cy, cv] = arr[i];
		for(int j = 0; j < n; j++) {
			if(i != j) {
				auto &[x, y, v] = arr[j];
				ld nx = x - cx, ny = y - cy;
				cur.emplace_back(atan2(ny, nx), v);
			}
		}
		sort(begin(cur), end(cur));
		auto tmp = cur;
		for(auto &a: tmp) {
			a.first += 2 * pi;
		}
		cur.insert(cur.end(), begin(tmp), end(tmp));
		int j = 0, csum = cv;
		for(auto &[angle, v]: cur) {
			dbg(angle, v);
			if(angle > pi) {
				break;
			}
			while(j < sz(cur) && cur[j].first <= angle + pi) {
				csum += cur[j].second;
				j++;
			}
			dbg(v, j);
			ans = max(ans, csum);
			csum -= v;
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