//package Mock_CCC_Senior21;

import java.util.*;
import java.io.*;

import static java.lang.Math.*;

public class Problem3 {
	static class Input {
		final byte[] buff = System.in.readAllBytes();
		int ind = 0;
		public Input() throws IOException {
		}
		public int nextInt() throws IOException {
			byte b;
			while((b = buff[ind++])<'0'||b>'9') ;
			int ret = b-'0';
			while((b = buff[ind++])>='0'&&b<='9') {
				ret = ret*10+(b-'0');
			}
			return ret;
		}
	}

	public static void main(String[] args) throws Exception {
		long start = System.currentTimeMillis();
		Input in = new Input();
		PrintWriter pw = new PrintWriter(System.out);
		new Problem3().solve(in, pw);
		pw.close();
		long end = System.currentTimeMillis();
		System.err.println(end-start);
	}

	static final boolean LOCAL = false;

	public static void dbg(Object... o) {
		if(LOCAL) {
			System.err.println(Arrays.deepToString(o));
		}
	}

//	static class ST {
//		int n;
//		int[] arr, v, child;
//		public ST(int[] arr) {
//			this.n = arr.length;
//			this.arr = arr;
//			v = new int[4*n];
//			child = new int[n];
//			build(1, 0, n);
//		}
//		public void build(int o, int l, int r) {
//			if(l+1==r) {
//				v[child[l] = o] = arr[l];
//			}else {
//				int mid = (l+r)/2, lc = o*2, rc = lc+1;
//				build(lc, l, mid);
//				build(rc, mid, r);
//				v[o] = min(v[lc], v[rc]);
//			}
//		}
//		public void set(int ind, int x) {
//			v[child[ind]] = x;
//			for(int i = child[ind]/2; i>=1; i/=2) {
//				v[i] = min(v[i*2], v[i*2+1]);
//			}
//		}
//		public int query(int x) {
//			int o = 1, l = 0, r = n;
//			while(l+1<r) {
//				int mid = (l+r)/2, lc = o*2;
//				if(v[lc]<=x) {
//					o = lc;
//					r = mid;
//				}else {
//					o = lc+1;
//					l = mid;
//				}
//			}
//			return l;
//		}
//	}

//	static class Deque {
//		int n, l, r;
//		int[] arr;
//		public Deque() {
//			n = 16;
//			l = r = 0;
//			arr = new int[n];
//		}
//		public void addLast(int x) {
//			if(r==n) {
//				arr = Arrays.copyOf(arr, n *= 2);
//			}
//			arr[r++] = x;
//		}
//		public int getFirst() {
//			return arr[l];
//		}
//		public void removeFirst() {
//			l++;
//		}
//		public boolean isEmpty() {
//			return l==r;
//		}
//	}

	public void solve(Input in, PrintWriter pw) throws Exception {
//		int n = in.nextInt(), k = in.nextInt();
//		Deque[] ind = new Deque[k];
//		for(int i = 0; i<k; i++) {
//			ind[i] = new Deque();
//		}
//		int[][] last = new int[k][2];
//		int[] arr = new int[n];
//		for(int i = 0; i<n; i++) {
//			ind[arr[i] = in.nextInt()-1].addLast(i);
//			last[arr[i]][0] = i;
//		}
//		for(int i = 0; i<k; i++) {
//			last[i][1] = i;
//		}
//		Arrays.sort(last, Comparator.comparingInt(o -> o[0]));
//		boolean[] removed = new boolean[k];
//		ST st;
//		{
//			int[] tmp = new int[k];
//			for(int i = 0; i<k; i++) {
//				tmp[i] = ind[i].getFirst();
//			}
//			st = new ST(tmp);
//		}
//		int prev = 0;
//		StringBuilder sb = new StringBuilder(k*4);
//		for(int[] i: last) {
//			if(!removed[i[1]]) {
//				while(true) {
//					int cur = st.query(i[0]);
//					sb.append(cur+1).append(" ");
//					removed[cur] = true;
//					st.set(cur, n+1);
//					int time = ind[cur].getFirst();
//					for(int j = prev; j<=time; j++) {
//						int tmp = arr[j];
//						ind[tmp].removeFirst();
//						if(!removed[tmp]) {
//							if(ind[tmp].isEmpty()) {
//								st.set(tmp, n+1);
//							}else {
//								st.set(tmp, ind[tmp].getFirst());
//							}
//						}
//					}
//					prev = time+1;
//					if(cur==i[1]) {
//						break;
//					}
//				}
//			}
//		}
//		System.out.println(sb.deleteCharAt(sb.length()-1).toString());
		int n = in.nextInt(), k = in.nextInt();
		int[] arr = new int[n], sort = new int[n];
		for(int i = 0; i<n; i++) {
			arr[i] = sort[i] = in.nextInt()-1;
		}
		int[][] last = new int[k][2];
		{
			int ind = k;
			boolean[] marked = new boolean[k];
			for(int i = n-1; i>=0; i--) {
				if(!marked[arr[i]]) {
					marked[arr[i]] = true;
					last[--ind][0] = i;
					last[ind][1] = arr[i];
				}
			}
		}
		int prev = 0;
		int[] cnt = new int[k];
		boolean[] removed = new boolean[k];
		StringBuilder sb = new StringBuilder(6*n);
		for(var v: last) {
			if(!removed[v[1]]) {
				int next;
				Arrays.sort(sort, prev, next = v[0]+1);
				int j = prev;
				for(int i = prev; i<next; i++) {
					cnt[arr[i]]++;
				}
				for(int i = prev; i<next; i++) {
					int cur = sort[i];
					if(!removed[cur]&&cnt[cur]>0) {
						for(; arr[j]!=cur; cnt[arr[j]]--, j++) ;
						cnt[cur]--;
						removed[cur] = true;
						sb.append(cur+1).append(' ');
					}
					if(cur==v[1]) {
						break;
					}
				}
				prev = next;
			}
		}
		System.out.println(sb.deleteCharAt(sb.length()-1).toString());
	}
}