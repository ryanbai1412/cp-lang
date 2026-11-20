#include <bits/stdc++.h>
using namespace std; using ll = long long;
int main(){ ll n; cin>>n; ll s=0; for(int r=0;r<20;r++) for(ll i=0;i<n;i+=2) s+=i%7; cout<<s<<"\n"; }
