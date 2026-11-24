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
}
