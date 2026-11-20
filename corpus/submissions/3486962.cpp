#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

bool valid(vector<int> arr, int k) {
	if(arr[0] == 0) {
		return true;
	}
	int n = sz(arr);
	int tot = 0;
	for(int j = 0; j < n; j++) {
		int x = 1;
		for(int a = 0; a < k; a++) {
			x *= arr[(j + a) % n];
		}
		tot += x;
	}
	return tot == 0;
}

vector<int> solve(int n, int k) {
	if(n & 1) {
		return {0};
	}else if(n == k) {
		return {0};
	}else if(n % 4 == 2 && (k & 1) == 0) {
		return {0};
	}
	if(k & 1) {
		vector<int> ans(n, 1);
		for(int i = 0; i < n; i++) {
			if(i & 1) {
				ans[i] = -1;
			}
		}
		return ans;
	}
	k = min(k, n - k);
	vector<int> ans(n, 1);
	int x = 0;
	while(x < n / 2) {
		ans[x] = -1;
		x += k;
	}
	ans[x - k / 2 + ((x - n / 2) / 2)] = -1;
	return ans;
}

mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());

int rint(int r) {
	return cowng() % r;
}

int rint(int l, int r) {
	return l + rint(r - l + 1);
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
//	for(int i = 4; i <= 28; i += 4) {
//		cout << i << endl;
//		for(int j = 1; j < i; j++) {
//			if(j < 10) {
//				cout << " ";
//			}
//			cout << j << " ";
//			auto ans = solve(i, j);
//			if(ans != vector<int>({0})) {
////				cout << i << " " << j << endl;
//				for(int x = 0; x < i; x++) {
//					if(ans[x] == 1) {
//						cout << " ";
//					}else {
//						cout << "#";
//					}
//				}
//				cout << endl;
//			}
//		}
//	}
	int n, k;
	cin >> n >> k;
	auto ans = solve(n, k);
	assert(valid(ans, k));
	for(int i = 0; i < sz(ans); i++) {
		if(i) {
			cout << " ";
		}
		cout << ans[i];
	}
	cout << endl;
}