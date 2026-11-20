#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int((x).size())

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, k;
	cin >> n >> k;
	for(int i = 0; i < 1 << n; i++) {
		int cur[n];
		for(int j = 0; j < n; j++) {
			if((i >> j) & 1) {
				cur[j] = -1;
			}else {
				cur[j] = 1;
			}
		}
		int tot = 0;
		for(int j = 0; j < n; j++) {
			int x = 1;
			for(int a = 0; a < k; a++) {
				x *= cur[(j + a) % n];
			}
			tot += x;
		}
		if(tot == 0) {
			for(int j = 0; j < n; j++) {
				if(j) {
					cout << " ";
				}
				cout << cur[j];
			}
			cout << endl;
			return 0;
		}
	}
	cout << "0" << endl;
}