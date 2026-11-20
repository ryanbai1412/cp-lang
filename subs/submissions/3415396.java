import java.io.BufferedOutputStream;
import java.io.Closeable;
import java.io.DataInputStream;
import java.io.Flushable;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.InputMismatchException;
import java.util.Objects;

/**
 * Built using CHelper plug-in
 * Actual solution is at the top
 */
public class Main {
	static class TaskAdapter implements Runnable {
		@Override
		public void run() {
			long startTime = System.currentTimeMillis();
			InputStream inputStream = System.in;
			OutputStream outputStream = System.out;
			FastReader in = new FastReader(inputStream);
			Output out = new Output(outputStream);
			RiolkusMockCCCS3MoseysBirthday solver = new RiolkusMockCCCS3MoseysBirthday();
			solver.solve(1, in, out);
			out.close();
			System.err.println(System.currentTimeMillis()-startTime+"ms");
		}
	}

	public static void main(String[] args) throws Exception {
		Thread thread = new Thread(null, new TaskAdapter(), "", 1<<29);
		thread.start();
		thread.join();
	}

	static class RiolkusMockCCCS3MoseysBirthday {
		int n;
		int cur;
		int ind;
		int[] color;
		int[] conv;
		boolean[] visited;
		ArrayList<Integer> comp;
		ArrayList<Integer>[] graph;
		ArrayList<Integer>[] comps;
		ArrayList<Integer>[] cgraph;
		ArrayList<Integer>[][] graphs;

		public RiolkusMockCCCS3MoseysBirthday() {
		}

		public void pdfs(int u) {
			comp.add(u);
			color[u] = cur;
			conv[u] = ind++;
			for(int v: graph[u]) {
				if(color[v]==-1) {
					pdfs(v);
				}
			}
		}

		public void dfs(int u) {
			visited[u] = true;
			for(int v: graph[u]) {
				if(!visited[v]) {
					cgraph[conv[u]].add(conv[v]);
					cgraph[conv[v]].add(conv[u]);
					dfs(v);
				}
			}
		}

		public void solve(int kase, InputReader in, Output pw) {
			n = in.nextInt();
			int m = in.nextInt();
			int[] arr = in.nextInt(n);
			graph = in.nextUndirectedGraph(n, m);
			cur = 0;
			color = new int[n];
			conv = new int[n];
			comps = new ArrayList[n];
			Arrays.fill(color, -1);
			for(int i = 0; i<n; i++) {
				if(color[i]==-1) {
					ind = 0;
					comp = new ArrayList<>();
					pdfs(i);
					comps[cur++] = comp;
				}
			}
			visited = new boolean[n];
			graphs = new ArrayList[n][];
			for(int i = 0; i<n; i++) {
				if(!visited[i]) {
					cgraph = new ArrayList[comps[color[i]].size()];
					for(int j = 0; j<comps[color[i]].size(); j++) {
						cgraph[j] = new ArrayList<>();
					}
					dfs(i);
					graphs[color[i]] = cgraph;
				}
			}
			ArrayList<Pair<Integer, Integer>> ans = new ArrayList<>();
			boolean[] marked = new boolean[n];
			for(int i = 1; i<=n; i++) {
				int ind = 0;
				for(int j = 0; j<n; j++) {
					if(arr[j]==i) {
						ind = j;
					}
				}
				int comp = color[ind];
				ArrayList<Integer>[] graph = graphs[comp];
				ArrayList<Integer> sorted = new ArrayList<>(comps[comp]);
				sorted.sort(null);
				for(int j: sorted) {
					if(!marked[j]) {
						marked[j] = true;
						int n = comps[comp].size();
						int[] from = new int[n];
						boolean[] visited = new boolean[n];
						ArrayDeque<Integer> s = new ArrayDeque<>();
						s.addLast(conv[j]);
						from[conv[j]] = -1;
						visited[conv[j]] = true;
						while(!s.isEmpty()) {
							int u = s.removeLast();
							for(int v: graph[u]) {
								if(!visited[v]) {
									visited[v] = true;
									from[v] = u;
									s.addLast(v);
								}
							}
						}
						int cur = conv[ind];
						while(cur!=conv[j]) {
							int u = comps[comp].get(cur), v = comps[comp].get(from[cur]);
							cur = from[cur];
							Utilities.swap(arr, u, v);
							ans.add(new Pair<>(u, v));
						}
						break;
					}
				}
			}
			if(ans.size()>1000000) {
				throw new AssertionError();
			}
			pw.println(ans.size());
			pw.println(arr);
			for(var v: ans) {
				pw.println(v.a+1, v.b+1);
			}
		}

	}

	static class Pair<T1, T2> implements Comparable<Pair<T1, T2>> {
		public T1 a;
		public T2 b;

		public Pair(Pair<T1, T2> p) {
			this(p.a, p.b);
		}

		public Pair(T1 a, T2 b) {
			this.a = a;
			this.b = b;
		}

		public String toString() {
			return Utilities.Debug.ts(a)+" "+Utilities.Debug.ts(b);
		}

		public int hashCode() {
			return Objects.hash(a, b);
		}

		public boolean equals(Object o) {
			if(o instanceof Pair) {
				Pair p = (Pair) o;
				return a.equals(p.a)&&b.equals(p.b);
			}
			return false;
		}

		public int compareTo(Pair<T1, T2> p) {
			int cmp = ((Comparable<T1>) a).compareTo(p.a);
			if(cmp==0) {
				return ((Comparable<T2>) b).compareTo(p.b);
			}
			return cmp;
		}

	}

	static class FastReader implements InputReader {
		final private int BUFFER_SIZE = 1<<16;
		private DataInputStream din;
		private byte[] buffer;
		private int bufferPointer;
		private int bytesRead;

		public FastReader(InputStream is) {
			din = new DataInputStream(is);
			buffer = new byte[BUFFER_SIZE];
			bufferPointer = bytesRead = 0;
		}

		public int nextInt() {
			int ret = 0;
			byte c = skipToDigit();
			boolean neg = (c=='-');
			if(neg) {
				c = read();
			}
			do {
				ret = ret*10+c-'0';
			} while((c = read())>='0'&&c<='9');
			if(neg) {
				return -ret;
			}
			return ret;
		}

		private boolean isDigit(byte b) {
			return b>='0'&&b<='9';
		}

		private byte skipToDigit() {
			byte ret;
			while(!isDigit(ret = read())&&ret!='-') ;
			return ret;
		}

		private void fillBuffer() {
			try {
				bytesRead = din.read(buffer, bufferPointer = 0, BUFFER_SIZE);
			}catch(IOException e) {
				e.printStackTrace();
				throw new InputMismatchException();
			}
			if(bytesRead==-1) {
				buffer[0] = -1;
			}
		}

		private byte read() {
			if(bytesRead==-1) {
				throw new InputMismatchException();
			}else if(bufferPointer==bytesRead) {
				fillBuffer();
			}
			return buffer[bufferPointer++];
		}

	}

	static class Utilities {
		public static void swap(int[] arr, int i, int j) {
			if(i!=j) {
				arr[i] ^= arr[j];
				arr[j] ^= arr[i];
				arr[i] ^= arr[j];
			}
		}

		public static class Debug {
			public static <T> String ts(T t) {
				if(t==null) {
					return "null";
				}
				if(t instanceof Iterable) {
					return ts((Iterable<?>) t);
				}else if(t instanceof int[]) {
					String s = Arrays.toString((int[]) t);
					return "{"+s.substring(1, s.length()-1)+"}";
				}else if(t instanceof long[]) {
					String s = Arrays.toString((long[]) t);
					return "{"+s.substring(1, s.length()-1)+"}";
				}else if(t instanceof char[]) {
					String s = Arrays.toString((char[]) t);
					return "{"+s.substring(1, s.length()-1)+"}";
				}else if(t instanceof double[]) {
					String s = Arrays.toString((double[]) t);
					return "{"+s.substring(1, s.length()-1)+"}";
				}else if(t instanceof boolean[]) {
					String s = Arrays.toString((boolean[]) t);
					return "{"+s.substring(1, s.length()-1)+"}";
				}else if(t instanceof Object[]) {
					return ts((Object[]) t);
				}
				return t.toString();
			}

			private static <T> String ts(T[] arr) {
				StringBuilder ret = new StringBuilder();
				ret.append("{");
				boolean first = true;
				for(T t: arr) {
					if(!first) {
						ret.append(", ");
					}
					first = false;
					ret.append(ts(t));
				}
				ret.append("}");
				return ret.toString();
			}

			private static <T> String ts(Iterable<T> iter) {
				StringBuilder ret = new StringBuilder();
				ret.append("{");
				boolean first = true;
				for(T t: iter) {
					if(!first) {
						ret.append(", ");
					}
					first = false;
					ret.append(ts(t));
				}
				ret.append("}");
				return ret.toString();
			}

		}

	}

	static class Output implements Closeable, Flushable {
		public StringBuilder sb;
		public OutputStream os;
		public int BUFFER_SIZE;
		public String lineSeparator;

		public Output(OutputStream os) {
			this(os, 1<<16);
		}

		public Output(OutputStream os, int bs) {
			BUFFER_SIZE = bs;
			sb = new StringBuilder(BUFFER_SIZE);
			this.os = new BufferedOutputStream(os, 1<<17);
			lineSeparator = System.lineSeparator();
		}

		public void print(String s) {
			sb.append(s);
			if(sb.length()>BUFFER_SIZE/2) {
				flushToBuffer();
			}
		}

		public void print(int i) {
			print(String.valueOf(i));
		}

		public void print(long l) {
			print(String.valueOf(l));
		}

		public void print(double d) {
			print(String.valueOf(d));
		}

		public void print(char[] c) {
			print(String.valueOf(c));
		}

		public void print(Object... o) {
			for(int i = 0; i<o.length; i++) {
				if(i!=0) {
					print(" ");
				}
				print(o[i]);
			}
		}

		public void print(Object o) {
			if(o==null) {
				print("null");
			}else if(o instanceof Iterable) {
				print((Iterable<?>) o);
			}else if(o instanceof int[]) {
				print((int[]) o);
			}else if(o instanceof long[]) {
				print((long[]) o);
			}else if(o instanceof char[]) {
				print((char[]) o);
			}else if(o instanceof double[]) {
				print((double[]) o);
			}else if(o instanceof Object[]) {
				print((Object[]) o);
			}else {
				print(o.toString());
			}
		}

		public void print(int[] arr) {
			for(int i = 0; i<arr.length; i++) {
				if(i!=0) {
					print(" ");
				}
				print(arr[i]);
			}
		}

		public void print(long[] arr) {
			for(int i = 0; i<arr.length; i++) {
				if(i!=0) {
					print(" ");
				}
				print(arr[i]);
			}
		}

		public void print(double[] arr) {
			for(int i = 0; i<arr.length; i++) {
				if(i!=0) {
					print(" ");
				}
				print(arr[i]);
			}
		}

		public <T> void print(Iterable<T> iter) {
			boolean first = true;
			for(T t: iter) {
				if(!first) {
					print(" ");
				}
				first = false;
				print(t);
			}
		}

		public void println(int i) {
			println(String.valueOf(i));
		}

		public void println() {
			sb.append(lineSeparator);
		}

		public void println(String s) {
			sb.append(s);
			println();
		}

		public void println(Object... o) {
			for(int i = 0; i<o.length; i++) {
				if(i!=0) {
					print(" ");
				}
				print(o[i]);
			}
			println();
		}

		public void println(int[] arr) {
			print(arr);
			println();
		}

		private void flushToBuffer() {
			try {
				os.write(sb.toString().getBytes());
			}catch(IOException e) {
				e.printStackTrace();
			}
			sb = new StringBuilder(BUFFER_SIZE);
		}

		public void flush() {
			try {
				flushToBuffer();
				os.flush();
			}catch(IOException e) {
				e.printStackTrace();
			}
		}

		public void close() {
			flush();
			try {
				os.close();
			}catch(IOException e) {
				e.printStackTrace();
			}
		}

	}

	static interface InputReader {
		int nextInt();

		default int[] nextInt(int n) {
			int[] ret = new int[n];
			for(int i = 0; i<n; i++) {
				ret[i] = nextInt();
			}
			return ret;
		}

		default ArrayList<Integer>[] nextUndirectedGraph(int n, int m) {
			ArrayList<Integer>[] ret = new ArrayList[n];
			for(int i = 0; i<n; i++) {
				ret[i] = new ArrayList<>();
			}
			for(int i = 0; i<m; i++) {
				int u = nextInt()-1, v = nextInt()-1;
				ret[u].add(v);
				ret[v].add(u);
			}
			return ret;
		}

	}
}