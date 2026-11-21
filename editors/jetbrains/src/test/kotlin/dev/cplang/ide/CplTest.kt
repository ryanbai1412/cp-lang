package dev.cplang.ide

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.io.File
import java.nio.file.Files

class CplTest {
    @Test
    fun `commands are the cpl subcommands`() {
        assertEquals(listOf("run", "a.cp", "i1"), Cpl.runArgs("run", "a.cp", "i1", ""))
        assertEquals(listOf("run", "a.cp", "--release"), Cpl.runArgs("run", "a.cp", "", "--release"))
        assertEquals(listOf("test", "a.cp"), Cpl.runArgs("test", "a.cp", "i1", ""))
        assertEquals(listOf("stress", "a.cp", "--seed", "a b"), Cpl.runArgs("stress", "a.cp", null, "--seed \"a b\""))
        assertEquals(listOf("run", "a.cp", "--trace", "--format", "html"), Cpl.traceArgs("a.cp", null))
        assertEquals(listOf("run", "a.cp", "--trace", "dfs", "--format", "html"), Cpl.traceArgs("a.cp", "dfs"))
        assertEquals(listOf("lsp"), Cpl.lspArgs())
    }

    @Test
    fun `samples come in the VS Code extension's order`() {
        val dir = Files.createTempDirectory("cpl").toFile()
        val source = File(dir, "a.cp").apply { writeText("") }
        for (name in listOf("i10", "i2", "o2", "notes", "a.tests/3.in", "a.tests/3.out", "a.tests/12.in", "a.samples/1.in", "b.tests/1.in")) {
            File(dir, name).apply { parentFile.mkdirs(); writeText("") }
        }
        val got = Cpl.samples(source).map { it.relativeTo(dir).path }
        assertEquals(listOf("i2", "i10", "a.tests/3.in", "a.tests/12.in", "a.samples/1.in"), got)
    }

    @Test
    fun `the fn at the caret is the one to trace`() {
        val text = "use x\n\n/// d\n@memo\nfn dfs(v: Int):\n    go(v)\n\n// c\nfn main():\n    dfs(0)\nlet top = 1\n"
        assertNull(Cpl.enclosingFn(text, text.indexOf("use")))
        assertEquals("dfs", Cpl.enclosingFn(text, text.indexOf("go(")))
        assertEquals("main", Cpl.enclosingFn(text, text.indexOf("dfs(0)")))
        assertNull(Cpl.enclosingFn(text, text.indexOf("top")))
    }
}
