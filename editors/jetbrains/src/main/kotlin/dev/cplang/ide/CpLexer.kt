package dev.cplang.ide

import com.intellij.lexer.LexerBase
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.TokenSet

class CpTokenType(debugName: String) : IElementType(debugName, CpLanguage)

object CpTokens {
    @JvmField val KEYWORD = CpTokenType("KEYWORD")
    @JvmField val IDENTIFIER = CpTokenType("IDENTIFIER")
    @JvmField val TYPE_NAME = CpTokenType("TYPE_NAME")
    @JvmField val NUMBER = CpTokenType("NUMBER")
    @JvmField val STRING = CpTokenType("STRING")
    @JvmField val CHAR = CpTokenType("CHAR")
    @JvmField val LABEL = CpTokenType("LABEL")
    @JvmField val LINE_COMMENT = CpTokenType("LINE_COMMENT")
    @JvmField val DOC_COMMENT = CpTokenType("DOC_COMMENT")
    @JvmField val DECORATOR = CpTokenType("DECORATOR")
    @JvmField val OPERATOR = CpTokenType("OPERATOR")
    @JvmField val COMMA = CpTokenType("COMMA")
    @JvmField val DOT = CpTokenType("DOT")
    @JvmField val LPAREN = CpTokenType("LPAREN")
    @JvmField val RPAREN = CpTokenType("RPAREN")
    @JvmField val LBRACKET = CpTokenType("LBRACKET")
    @JvmField val RBRACKET = CpTokenType("RBRACKET")
    @JvmField val LBRACE = CpTokenType("LBRACE")
    @JvmField val RBRACE = CpTokenType("RBRACE")

    @JvmField val COMMENTS = TokenSet.create(LINE_COMMENT, DOC_COMMENT)
    @JvmField val STRINGS = TokenSet.create(STRING, CHAR)

    /** The reserved words of `TokenKind::keyword` in `crates/cpl-syntax/src/token.rs`. */
    val KEYWORDS: Set<String> = setOf(
        "let", "var", "fn", "struct", "enum", "domain", "type", "const", "trait", "impl", "use", "pub",
        "if", "elif", "else", "for", "while", "in", "match", "return", "break", "continue",
        "and", "or", "not", "assert", "mut", "self", "true", "false", "none", "lends", "yield", "unchecked",
    )
}

/**
 * The highlighting lexer. No token spans a line, so every line starts in state 0
 * and the editor can relex from any line.
 */
class CpLexer : LexerBase() {
    private var buffer: CharSequence = ""
    private var end = 0
    private var start = 0
    private var stop = 0
    private var type: IElementType? = null

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        this.buffer = buffer
        this.end = endOffset
        this.stop = startOffset
        advance()
    }

    override fun getState() = 0
    override fun getTokenType() = type
    override fun getTokenStart() = start
    override fun getTokenEnd() = stop
    override fun getBufferSequence() = buffer
    override fun getBufferEnd() = end

    override fun advance() {
        start = stop
        if (start >= end) {
            type = null
            return
        }
        type = scan()
    }

    private fun at(i: Int): Char = if (i < end) buffer[i] else '\u0000'

    private fun identStart(c: Char) = c == '_' || c.isLetter()
    private fun identPart(c: Char) = c == '_' || c.isLetterOrDigit()

    private fun scan(): IElementType {
        val c = buffer[start]
        var i = start + 1
        when {
            c == ' ' || c == '\t' || c == '\n' || c == '\r' -> {
                while (i < end && at(i).let { it == ' ' || it == '\t' || it == '\n' || it == '\r' }) i++
                stop = i
                return TokenType.WHITE_SPACE
            }
            c == '/' && at(i) == '/' -> {
                val doc = at(i + 1) == '/' && at(i + 2) != '/'
                while (i < end && at(i) != '\n') i++
                stop = i
                return if (doc) CpTokens.DOC_COMMENT else CpTokens.LINE_COMMENT
            }
            c == 'r' && at(i) == '"' -> {
                i++
                while (i < end && at(i) != '"' && at(i) != '\n') i++
                if (at(i) == '"') i++
                stop = i
                return CpTokens.STRING
            }
            c == '"' -> {
                while (i < end && at(i) != '"' && at(i) != '\n') i += if (at(i) == '\\' && at(i + 1) != '\n') 2 else 1
                if (at(i) == '"') i++
                stop = minOf(i, end)
                return CpTokens.STRING
            }
            c == '\'' -> {
                if (at(i) == '\\' && at(i + 2) == '\'') {
                    stop = i + 3
                    return CpTokens.CHAR
                }
                if (at(i) != '\n' && at(i) != '\u0000' && at(i + 1) == '\'') {
                    stop = i + 2
                    return CpTokens.CHAR
                }
                if (identStart(at(i))) {
                    while (identPart(at(i))) i++
                    stop = i
                    return CpTokens.LABEL
                }
                stop = i
                return CpTokens.OPERATOR
            }
            c == '@' && identStart(at(i)) -> {
                while (identPart(at(i))) i++
                stop = i
                return CpTokens.DECORATOR
            }
            c.isDigit() -> {
                stop = number(start)
                return CpTokens.NUMBER
            }
            identStart(c) -> {
                while (identPart(at(i))) i++
                stop = i
                val word = buffer.subSequence(start, i).toString()
                return when {
                    word in CpTokens.KEYWORDS -> CpTokens.KEYWORD
                    c.isUpperCase() -> CpTokens.TYPE_NAME
                    else -> CpTokens.IDENTIFIER
                }
            }
        }
        stop = i
        return when (c) {
            ',' -> CpTokens.COMMA
            '.' -> CpTokens.DOT
            '(' -> CpTokens.LPAREN
            ')' -> CpTokens.RPAREN
            '[' -> CpTokens.LBRACKET
            ']' -> CpTokens.RBRACKET
            '{' -> CpTokens.LBRACE
            '}' -> CpTokens.RBRACE
            else -> CpTokens.OPERATOR
        }
    }

    private fun number(from: Int): Int {
        var i = from
        if (at(i) == '0' && (at(i + 1) == 'x' || at(i + 1) == 'X' || at(i + 1) == 'b' || at(i + 1) == 'B')) {
            i += 2
            while (at(i).isLetterOrDigit() || at(i) == '_') i++
            return i
        }
        while (at(i).isDigit() || at(i) == '_') i++
        // `1.5` is a float; `0..n` and `xs.0.len` are not.
        if (at(i) == '.' && at(i + 1).isDigit()) {
            i++
            while (at(i).isDigit() || at(i) == '_') i++
        }
        if ((at(i) == 'e' || at(i) == 'E') && (at(i + 1).isDigit() || ((at(i + 1) == '+' || at(i + 1) == '-') && at(i + 2).isDigit()))) {
            i += 2
            while (at(i).isDigit() || at(i) == '_') i++
        }
        return i
    }
}
