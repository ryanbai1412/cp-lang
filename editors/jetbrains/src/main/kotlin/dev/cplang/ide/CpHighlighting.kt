package dev.cplang.ide

import com.intellij.lang.ASTNode
import com.intellij.lang.BracePair
import com.intellij.lang.Commenter
import com.intellij.lang.PairedBraceMatcher
import com.intellij.lang.ParserDefinition
import com.intellij.lang.PsiParser
import com.intellij.extapi.psi.ASTWrapperPsiElement
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors as C
import com.intellij.openapi.editor.HighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.colors.TextAttributesKey.createTextAttributesKey
import com.intellij.openapi.fileTypes.SingleLazyInstanceSyntaxHighlighterFactory
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase
import com.intellij.openapi.options.colors.AttributesDescriptor
import com.intellij.openapi.options.colors.ColorDescriptor
import com.intellij.openapi.options.colors.ColorSettingsPage
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.IFileElementType

object CpColors {
    @JvmField val KEYWORD = createTextAttributesKey("CP_KEYWORD", C.KEYWORD)
    @JvmField val TYPE_NAME = createTextAttributesKey("CP_TYPE_NAME", C.CLASS_NAME)
    @JvmField val IDENTIFIER = createTextAttributesKey("CP_IDENTIFIER", C.IDENTIFIER)
    @JvmField val NUMBER = createTextAttributesKey("CP_NUMBER", C.NUMBER)
    @JvmField val STRING = createTextAttributesKey("CP_STRING", C.STRING)
    @JvmField val LABEL = createTextAttributesKey("CP_LABEL", C.LABEL)
    @JvmField val LINE_COMMENT = createTextAttributesKey("CP_LINE_COMMENT", C.LINE_COMMENT)
    @JvmField val DOC_COMMENT = createTextAttributesKey("CP_DOC_COMMENT", C.DOC_COMMENT)
    @JvmField val DECORATOR = createTextAttributesKey("CP_DECORATOR", C.METADATA)
    @JvmField val OPERATOR = createTextAttributesKey("CP_OPERATOR", C.OPERATION_SIGN)
    @JvmField val COMMA = createTextAttributesKey("CP_COMMA", C.COMMA)
    @JvmField val DOT = createTextAttributesKey("CP_DOT", C.DOT)
    @JvmField val PARENTHESES = createTextAttributesKey("CP_PARENTHESES", C.PARENTHESES)
    @JvmField val BRACKETS = createTextAttributesKey("CP_BRACKETS", C.BRACKETS)
    @JvmField val BRACES = createTextAttributesKey("CP_BRACES", C.BRACES)
    @JvmField val BAD = createTextAttributesKey("CP_BAD_CHARACTER", HighlighterColors.BAD_CHARACTER)
}

class CpSyntaxHighlighter : SyntaxHighlighterBase() {
    override fun getHighlightingLexer() = CpLexer()

    override fun getTokenHighlights(type: IElementType?): Array<TextAttributesKey> = pack(
        when (type) {
            CpTokens.KEYWORD -> CpColors.KEYWORD
            CpTokens.TYPE_NAME -> CpColors.TYPE_NAME
            CpTokens.IDENTIFIER -> CpColors.IDENTIFIER
            CpTokens.NUMBER -> CpColors.NUMBER
            CpTokens.STRING, CpTokens.CHAR -> CpColors.STRING
            CpTokens.LABEL -> CpColors.LABEL
            CpTokens.LINE_COMMENT -> CpColors.LINE_COMMENT
            CpTokens.DOC_COMMENT -> CpColors.DOC_COMMENT
            CpTokens.DECORATOR -> CpColors.DECORATOR
            CpTokens.OPERATOR -> CpColors.OPERATOR
            CpTokens.COMMA -> CpColors.COMMA
            CpTokens.DOT -> CpColors.DOT
            CpTokens.LPAREN, CpTokens.RPAREN -> CpColors.PARENTHESES
            CpTokens.LBRACKET, CpTokens.RBRACKET -> CpColors.BRACKETS
            CpTokens.LBRACE, CpTokens.RBRACE -> CpColors.BRACES
            TokenType.BAD_CHARACTER -> CpColors.BAD
            else -> null
        },
    )
}

class CpSyntaxHighlighterFactory : SingleLazyInstanceSyntaxHighlighterFactory() {
    override fun createHighlighter() = CpSyntaxHighlighter()
}

class CpColorSettingsPage : ColorSettingsPage {
    private val descriptors = arrayOf(
        AttributesDescriptor("Keyword", CpColors.KEYWORD),
        AttributesDescriptor("Type name", CpColors.TYPE_NAME),
        AttributesDescriptor("Identifier", CpColors.IDENTIFIER),
        AttributesDescriptor("Number", CpColors.NUMBER),
        AttributesDescriptor("String and char", CpColors.STRING),
        AttributesDescriptor("Label", CpColors.LABEL),
        AttributesDescriptor("Comment", CpColors.LINE_COMMENT),
        AttributesDescriptor("Doc comment", CpColors.DOC_COMMENT),
        AttributesDescriptor("Decorator", CpColors.DECORATOR),
        AttributesDescriptor("Operator", CpColors.OPERATOR),
        AttributesDescriptor("Braces and brackets//Parentheses", CpColors.PARENTHESES),
        AttributesDescriptor("Braces and brackets//Brackets", CpColors.BRACKETS),
        AttributesDescriptor("Braces and brackets//Braces", CpColors.BRACES),
    )

    override fun getIcon() = CpIcons.FILE
    override fun getHighlighter() = CpSyntaxHighlighter()
    override fun getAdditionalHighlightingTagToDescriptorMap(): Map<String, TextAttributesKey>? = null
    override fun getAttributeDescriptors() = descriptors
    override fun getColorDescriptors(): Array<ColorDescriptor> = ColorDescriptor.EMPTY_ARRAY
    override fun getDisplayName() = "CP Lang"
    override fun getDemoText() = """
        /// The sum of everything left of the boundary.
        @memo
        fn ways(n: Int) -> Mod<1000000007>:
            if n <= 1:
                return 1
            ways(n - 1) + ways(n - 2)

        fn main():
            let (n, k) = read()
            var best = 0x7f
            for i in 0..n:
                best = max(best, k * i)  // no overflow: the types say so
            print(f"{best} {ways(n)}", 'x')
    """.trimIndent()
}

class CpParserDefinition : ParserDefinition {
    override fun createLexer(project: Project?) = CpLexer()

    override fun createParser(project: Project?) = PsiParser { root, builder ->
        val file = builder.mark()
        while (!builder.eof()) builder.advanceLexer()
        file.done(root)
        builder.treeBuilt
    }

    override fun getFileNodeType() = FILE
    override fun getCommentTokens() = CpTokens.COMMENTS
    override fun getStringLiteralElements() = CpTokens.STRINGS
    override fun createElement(node: ASTNode): PsiElement = ASTWrapperPsiElement(node)
    override fun createFile(viewProvider: FileViewProvider): PsiFile = CpFile(viewProvider)

    companion object {
        @JvmField val FILE = IFileElementType(CpLanguage)
    }
}

class CpCommenter : Commenter {
    override fun getLineCommentPrefix() = "// "
    override fun getBlockCommentPrefix(): String? = null
    override fun getBlockCommentSuffix(): String? = null
    override fun getCommentedBlockCommentPrefix(): String? = null
    override fun getCommentedBlockCommentSuffix(): String? = null
}

class CpBraceMatcher : PairedBraceMatcher {
    private val pairs = arrayOf(
        BracePair(CpTokens.LPAREN, CpTokens.RPAREN, false),
        BracePair(CpTokens.LBRACKET, CpTokens.RBRACKET, false),
        BracePair(CpTokens.LBRACE, CpTokens.RBRACE, false),
    )

    override fun getPairs() = pairs
    override fun isPairedBracesAllowedBeforeType(lbraceType: IElementType, contextType: IElementType?) = true
    override fun getCodeConstructStart(file: PsiFile?, openingBraceOffset: Int) = openingBraceOffset
}

