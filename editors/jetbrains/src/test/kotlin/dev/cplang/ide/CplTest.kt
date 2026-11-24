package dev.cplang.ide

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class CplTest {
    @Test
    fun `commands are the cpl subcommands`() {
        assertEquals(listOf("lsp"), Cpl.lspArgs())
    }

    @Test
    fun `the view a cpl report gives is drawn as is`() {
        val json = """{"status":"fail","view":{"panel":"FAIL\n","lines":[],"diagnostic":{"line":3,"message":"first wrong call: `tri(k: 1)`, the 1st call of `tri`","source":"cpl"}}}"""
        assertEquals(Cpl.View("FAIL\n", Cpl.WrongCall(3, "first wrong call: `tri(k: 1)`, the 1st call of `tri`")), Cpl.view(json))
        assertEquals(Cpl.View("ok\n", null), Cpl.view("""{"view":{"panel":"ok\n","lines":[]}}"""))
        assertNull(Cpl.view("error: no such file"))
    }
}
