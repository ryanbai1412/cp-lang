package dev.cplang.ide

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class CplTest {
    @Test
    fun `the installed cpl is found without PATH`() {
        val unix = Cpl.installed("/home/u", null, windows = false).map { it.path.replace('\\', '/') }
        assertEquals(listOf("/home/u/.cpl/bin/cpl", "/home/u/.cargo/bin/cpl"), unix)
        val moved = Cpl.installed("/home/u", "/opt/cpl", windows = true).map { it.path.replace('\\', '/') }
        assertEquals(listOf("/opt/cpl/bin/cpl.exe", "/home/u/.cpl/bin/cpl.exe", "/home/u/.cargo/bin/cpl.exe"), moved)
    }

    @Test
    fun `commands are the cpl subcommands`() {
        assertEquals(listOf("lsp"), Cpl.lspArgs())
    }

    @Test
    fun `importing a problem runs cpl new --listen and opens what it created`() {
        assertEquals(listOf("new", "--listen", "--json", "--port", "10043"), Cpl.companionArgs(COMPANION_PORT))
        val out = "{\"path\":\"cf1A.cp\",\"created\":true,\"tests\":\"cf1A.tests\",\"samples\":2}\n" +
            "not json\n{\"path\":\"cf1B.cp\",\"created\":true,\"tests\":\"cf1B.tests\",\"samples\":1}\n"
        assertEquals(listOf("cf1A.cp", "cf1B.cp"), Cpl.createdFiles(out))
    }

    @Test
    fun `the view a cpl report gives is drawn as is`() {
        val json = """{"status":"fail","view":{"panel":"FAIL\n","lines":[],"diagnostic":{"line":3,"message":"first wrong call: `tri(k: 1)`, the 1st call of `tri`","source":"cpl"}}}"""
        assertEquals(Cpl.View("FAIL\n", Cpl.WrongCall(3, "first wrong call: `tri(k: 1)`, the 1st call of `tri`")), Cpl.view(json))
        assertEquals(Cpl.View("ok\n", null), Cpl.view("""{"view":{"panel":"ok\n","lines":[]}}"""))
        assertNull(Cpl.view("error: no such file"))
    }

    @Test
    fun `the trace viewer's messages are a step or a clicked line`() {
        assertEquals(TraceMessage(false, 3), TraceMessage.parse("""{"type":"step","line":3}"""))
        assertEquals(TraceMessage(true, 1), TraceMessage.parse("""{"type":"reveal","line":1}"""))
        for (bad in listOf("""{"type":"step","line":0}""", """{"type":"step","line":1.5}""", """{"type":"step","line":"2"}""",
                """{"type":"run","line":2}""", """{"line":2}""", "[1]", "nope")) {
            assertNull(bad, TraceMessage.parse(bad))
        }
    }

    @Test
    fun `the bridge comes before the viewer's own scripts`() {
        val html = CplTrace.bridge("<!doctype html>\n<HTML><head lang=x><script>viewer()</script></head></HTML>", "send(json);")
        val shim = html.indexOf("window.acquireVsCodeApi")
        assertTrue(html, shim > html.indexOf("<head lang=x>") && shim < html.indexOf("viewer()"))
        assertTrue(html.contains("const json = JSON.stringify(message); send(json);"))
        assertTrue(CplTrace.bridge("<p>x</p>", "s(json)").startsWith("<script>window.acquireVsCodeApi"))
    }
}
