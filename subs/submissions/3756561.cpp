#pragma GCC optimize("Ofast,unroll-loops,no-stack-protector")
#pragma GCC target("mmx,sse,sse2,sse3,ssse3,sse4,sse4a,sse4.1,sse4.2,avx,avx2,fma,fma4,popcnt,abm,bmi,bmi2")

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
	int n, m;
	cin >> n >> m;
	assert(m <= INT_MAX);
	int arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	int q;
	cin >> q;
	while(q--) {
		string t;
		cin >> t;
		if(t == "R") {
			int k;
			cin >> k;
			int narr[n];
			for(int i = 0; i < n; i++) {
				narr[(i + k) % n] = arr[i];
			}
			memcpy(arr, narr, sizeof(arr));
		}else if(t == "F") {
			reverse(arr + 1, arr + n);
		}else if(t == "S") {
			int i, j;
			cin >> i >> j;
			i--;
			j--;
			swap(arr[i], arr[j]);
		}else if(t == "P") {
			int i, j, x;
			cin >> i >> j >> x;
			i--;
			j--;
			if(i <= j) {
				fill(arr + i, arr + j + 1, x);
			}else {
				fill(arr + i, arr + n, x);
				fill(arr, arr + j + 1, x);
			}
		}else if(t == "C") {
			if(n == 1) {
				cout << 1 << endl;
				return;
			}
			int ans = 1;
			for(int i = 0; i < n - 1; i++) {
				ans += arr[i] != arr[i + 1];
			}
			if(n > 2 && arr[0] == arr[n - 1]) {
				ans--;
			}
			cout << ans << endl;
		}else {
			int l, r;
			cin >> l >> r;
			l--;
			r--;
			int ans = 1;
			for(int i = l; i < r; i++) {
				ans += arr[i] != arr[i + 1];
			}
			cout << ans << endl;
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
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}