package dev.cplang.ide

import com.intellij.lexer.LexerBase
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.colors.TextAttributesKey.createTextAttributesKey
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.TokenSet
import java.util.regex.Pattern

class CpTokenType(debugName: String) : IElementType(debugName, CpLanguage)

/**
 * The tokens, colors and rules of editors/lexer.tsv, which `cpl setup jetbrains --grammar`
 * generates from cpl-syntax's token tables: per line a token, its color key, the key's
 * default, its name on the color settings page, and the regex.
 */
object CpTokens {
    class Rule(val token: IElementType, val color: TextAttributesKey?, val label: String, val pattern: Pattern)

    private val tokens = mutableMapOf<String, IElementType>("WHITE_SPACE" to TokenType.WHITE_SPACE)

    val RULES: List<Rule> = CpTokens::class.java.getResource("/cp/lexer.tsv")!!.readText().lines()
        .filter { it.isNotBlank() && !it.startsWith("#") }
        .map { it.split('\t', limit = 5) }
        .map { (name, key, default, label, regex) ->
            val color = key.takeIf { it != "-" }?.let {
                createTextAttributesKey("CP_$it", DefaultLanguageHighlighterColors::class.java.getField(default).get(null) as TextAttributesKey)
            }
            Rule(tokens.getOrPut(name) { CpTokenType(name) }, color, label, Pattern.compile(regex))
        }

    val COLORS: Map<IElementType, TextAttributesKey> = RULES.mapNotNull { r -> r.color?.let { r.token to it } }.toMap()

    fun named(name: String): IElementType = tokens.getValue(name)

    val COMMENTS = TokenSet.create(named("LINE_COMMENT"), named("DOC_COMMENT"))
    val STRINGS = TokenSet.create(named("STRING"), named("CHAR"))
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
        for (rule in CpTokens.RULES) {
            val m = rule.pattern.matcher(buffer).region(start, end).useTransparentBounds(true)
            if (m.lookingAt() && m.end() > start) {
                stop = m.end()
                type = rule.token
                return
            }
        }
    }
}
