import java.io.*;

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
		public RiolkusMockCCCS3MoseysBirthday() {
		}

		public void solve(int kase, InputReader in, Output pw) {
			throw new AssertionError();
//		n = in.nextInt();
//		int m = in.nextInt();
//		arr = in.nextInt(n);
//		graph = in.nextUndirectedGraph(n, m);
//		cur = 0;
//		color = new int[n];
//		Arrays.fill(color, -1);
//		vals = new ArrayList[n];
//		for(int i = 0; i<n; i++) {
//			if(color[i]==-1) {
//				vals[cur] = new ArrayList<>();
//				pdfs(i);
//				cur++;
//			}
//		}
//		visited = new boolean[n];
//		graphs = new ArrayList[n][];
//		for(int i = 0; i<n; i++) {
//			if(!visited[i]) {
//				cgraph = new ArrayList[n];
//				for(int j = 0; j<n; j++) {
//					cgraph[j] = new ArrayList<>();
//				}
//				dfs(i);
//				graphs[color[i]] = cgraph;
//			}
//		}
//		ans = new ArrayList<>();
//		for(int i = 0; i<cur; i++) {
//			ArrayList<Integer> values = new ArrayList<>();
//			for(int j: vals[i]) {
//				values.add(arr[j]);
//			}
//			values.sort(null);
//			ArrayList<Integer> sorted = new ArrayList<>(vals[i]);
//			sorted.sort(null);
//			graph = graphs[i];
//			order = new ArrayList<>();
//			dfs(0, -1);
//			for(int j: order) {
//				int ind = sorted.indexOf(j);
//				int should = values.get(ind);
//				for(int k = 0; k<n; k++) {
//					if(arr[k]==should) {
//						move(k, j);
//					}
//				}
//			}
//		}
//		if(ans.size()>1000000) {
//			throw new AssertionError();
//		}
//		pw.println(ans.size());
//		pw.println(arr);
//		for(var v: ans) {
//			pw.println(v.a+1, v.b+1);
//		}
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

	interface InputReader {
	}

	static class FastReader implements InputReader {
		final private int BUFFER_SIZE = 1<<16;
		private final DataInputStream din;
		private final byte[] buffer;
		private final int bufferPointer;
		private final int bytesRead;

		public FastReader(InputStream is) {
			din = new DataInputStream(is);
			buffer = new byte[BUFFER_SIZE];
			bufferPointer = bytesRead = 0;
		}

	}
}