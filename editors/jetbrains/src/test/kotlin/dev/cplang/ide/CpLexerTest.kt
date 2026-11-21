package dev.cplang.ide

import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File

class CpLexerTest {
    private val repo = File(System.getProperty("cpl.repo"))

    private fun lex(text: String): List<Pair<IElementType, String>> {
        val lexer = CpLexer()
        lexer.start(text, 0, text.length, 0)
        val out = mutableListOf<Pair<IElementType, String>>()
        var at = 0
        while (lexer.tokenType != null) {
            assertEquals("tokens are contiguous", at, lexer.tokenStart)
            assertTrue("tokens are not empty", lexer.tokenEnd > lexer.tokenStart)
            assertEquals("no token spans a line", 0, lexer.state)
            out += lexer.tokenType!! to text.substring(lexer.tokenStart, lexer.tokenEnd)
            at = lexer.tokenEnd
            lexer.advance()
        }
        assertEquals("the tokens cover the text", text.length, at)
        return out.filter { it.first != TokenType.WHITE_SPACE }
    }

    @Test
    fun `the keywords are the compiler's`() {
        val token = File(repo, "crates/cpl-syntax/src/token.rs").readText()
        val body = token.substringAfter("pub fn keyword(text: &str)").substringBefore("_ => return")
        val compiler = Regex("\"([a-z]+)\" =>").findAll(body).map { it.groupValues[1] }.toSortedSet()
        assertEquals(compiler, CpTokens.KEYWORDS.toSortedSet())
    }

    @Test
    fun `each token class lexes`() {
        val tokens = lex("/// doc\n@memo fn f(x: Int) -> [Pos]Char: // c\n    let s = r\"a\\b\" + \"q\\\"{x}\" + 'c' + '\\n'\n    for 'outer i in 0..n: xs.0 = 1.5e3 + 0x1F_ff")
        val kinds = tokens.map { it.first.toString() to it.second }
        val want = listOf(
            "DOC_COMMENT" to "/// doc", "DECORATOR" to "@memo", "KEYWORD" to "fn", "IDENTIFIER" to "f",
            "LPAREN" to "(", "IDENTIFIER" to "x", "OPERATOR" to ":", "TYPE_NAME" to "Int", "RPAREN" to ")",
            "OPERATOR" to "-", "OPERATOR" to ">", "LBRACKET" to "[", "TYPE_NAME" to "Pos", "RBRACKET" to "]",
            "TYPE_NAME" to "Char", "OPERATOR" to ":", "LINE_COMMENT" to "// c",
            "KEYWORD" to "let", "IDENTIFIER" to "s", "OPERATOR" to "=", "STRING" to "r\"a\\b\"", "OPERATOR" to "+",
            "STRING" to "\"q\\\"{x}\"", "OPERATOR" to "+", "CHAR" to "'c'", "OPERATOR" to "+", "CHAR" to "'\\n'",
            "KEYWORD" to "for", "LABEL" to "'outer", "IDENTIFIER" to "i", "KEYWORD" to "in", "NUMBER" to "0",
            "DOT" to ".", "DOT" to ".", "IDENTIFIER" to "n", "OPERATOR" to ":", "IDENTIFIER" to "xs", "DOT" to ".",
            "NUMBER" to "0", "OPERATOR" to "=", "NUMBER" to "1.5e3", "OPERATOR" to "+", "NUMBER" to "0x1F_ff",
        )
        assertEquals(want, kinds)
    }

    @Test
    fun `an unclosed string stops at the end of its line`() {
        val tokens = lex("let s = \"open\nlet t = 1")
        assertEquals("STRING" to "\"open", tokens[3].first.toString() to tokens[3].second)
        assertEquals("KEYWORD" to "let", tokens[4].first.toString() to tokens[4].second)
    }

    @Test
    fun `every stdlib and example file lexes with no gap`() {
        val files = listOf("stdlib", "examples", "crates/cpl-cli/tests").flatMap { dir ->
            File(repo, dir).walkTopDown().filter { it.extension == "cp" }.toList()
        }
        assertTrue("found .cp files", files.size > 20)
        for (file in files) lex(file.readText())
    }
}
