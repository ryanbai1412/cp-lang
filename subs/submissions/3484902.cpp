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
	int t;
	cin >> t;
	while(t--) {
		//why can't I read
//		int n;
//		long x;
//		cin >> n >> x;
//		long sum = 0;
//		map<long, int> s;
//		long arr[n];
//		for(int i = 0; i < n; i++) {
//			cin >> arr[i];
//			s[arr[i]]++;
//			sum += arr[i];
//		}
//		set<pair<long, long>> ans;
//		for(int i = 0; i < n; i++) {
//			s[arr[i]]--;
//			int other = sum - x - arr[i];
//			if(arr[i] <= other && s[other] > 0){
//				ans.emplace(arr[i], other);
//			}
//			s[arr[i]]++;
//		}
//		cout << sz(ans) << endl;
		long n, x;
		cin >> n >> x;
		long tot = n * (n + 1) / 2;
		long req = tot - x;
		cout << max(0LL, (req - 1) / 2) << endl;
	}
}