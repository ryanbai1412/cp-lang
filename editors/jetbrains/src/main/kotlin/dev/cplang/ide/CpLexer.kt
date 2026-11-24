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
}

/**
 * The highlighting lexer: the rules of editors/lexer.tsv, which `cpl setup jetbrains --grammar`
 * generates from cpl-syntax's token tables, tried in order at each offset. No rule spans a line,
 * so every line starts in state 0 and the editor can relex from any line.
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
        type = null
        if (start >= end) return
        for ((token, pattern) in RULES) {
            val m = pattern.matcher(buffer).region(start, end).useTransparentBounds(true)
            if (m.lookingAt() && m.end() > start) {
                stop = m.end()
                type = token
                return
            }
        }
    }

    companion object {
        private val TOKENS: Map<String, IElementType> = CpTokens::class.java.fields
            .mapNotNull { f -> (f.get(null) as? IElementType)?.let { f.name to it } }
            .toMap() + ("WHITE_SPACE" to TokenType.WHITE_SPACE)

        val RULES: List<Pair<IElementType, java.util.regex.Pattern>> =
            CpLexer::class.java.getResource("/cp/lexer.tsv")!!.readText().lines()
                .filter { it.isNotBlank() && !it.startsWith("#") }
                .map { line -> line.substringBefore('\t').let(TOKENS::getValue) to java.util.regex.Pattern.compile(line.substringAfter('\t')) }
    }
}
