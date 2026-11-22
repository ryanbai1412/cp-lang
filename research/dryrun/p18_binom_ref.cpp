#include <bits/stdc++.h>
using namespace std;
const long long M=1e9+7;long long pw(long long b,long long e){long long r=1;b%=M;while(e){if(e&1)r=r*b%M;b=b*b%M;e>>=1;}return r;}
int main(){int n;cin>>n;while(n--){long long a,b;cin>>a>>b;if(b>a){puts("0");continue;}long long num=1,den=1;for(long long i=0;i<b;i++){num=num*((a-i)%M)%M;den=den*((i+1)%M)%M;}printf("%lld\n",num*pw(den,M-2)%M);}}
