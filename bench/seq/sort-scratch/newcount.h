#include <cstdlib>
#include <cstdio>
#include <new>
static size_t g_n=0,g_b=0,g_big=0;
void* operator new(size_t s){ g_n++; g_b+=s; if(s>=1<<20){g_big++; fprintf(stderr,"alloc %zu\n",s);} void*p=malloc(s); if(!p) throw std::bad_alloc(); return p;}
void operator delete(void*p) noexcept { free(p);} void operator delete(void*p,size_t) noexcept { free(p);}
struct Rep{~Rep(){fprintf(stderr,"news=%zu bytes=%zu big=%zu\n",g_n,g_b,g_big);}} g_rep;
