package dev.cplang.ide

import com.intellij.lang.ASTNode
import com.intellij.lang.BracePair
import com.intellij.lang.Commenter
import com.intellij.lang.PairedBraceMatcher
import com.intellij.lang.ParserDefinition
import com.intellij.lang.PsiParser
import com.intellij.extapi.psi.ASTWrapperPsiElement
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
    @JvmField val BAD = createTextAttributesKey("CP_BAD_CHARACTER", HighlighterColors.BAD_CHARACTER)
}

/** Each token's color is the key editors/lexer.tsv names for it (see [CpTokens]). */
class CpSyntaxHighlighter : SyntaxHighlighterBase() {
    override fun getHighlightingLexer() = CpLexer()

    override fun getTokenHighlights(type: IElementType?): Array<TextAttributesKey> =
        pack(if (type == TokenType.BAD_CHARACTER) CpColors.BAD else CpTokens.COLORS[type])
}

class CpSyntaxHighlighterFactory : SingleLazyInstanceSyntaxHighlighterFactory() {
    override fun createHighlighter() = CpSyntaxHighlighter()
}

class CpColorSettingsPage : ColorSettingsPage {
    private val descriptors = CpTokens.RULES
        .mapNotNull { r -> r.color?.let { AttributesDescriptor(r.label, it) } }
        .distinctBy { it.key }
        .toTypedArray()

    override fun getIcon() = CpIcons.FILE
    override fun getHighlighter() = CpSyntaxHighlighter()
    override fun getAdditionalHighlightingTagToDescriptorMap(): Map<String, TextAttributesKey>? = null
    override fun getAttributeDescriptors() = descriptors
    override fun getColorDescriptors(): Array<ColorDescriptor> = ColorDescriptor.EMPTY_ARRAY
    override fun getDisplayName() = "CP Lang"
    override fun getDemoText() = CpColorSettingsPage::class.java.getResource("/cp/demo.cp")!!.readText()
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
        BracePair(CpTokens.named("LPAREN"), CpTokens.named("RPAREN"), false),
        BracePair(CpTokens.named("LBRACKET"), CpTokens.named("RBRACKET"), false),
        BracePair(CpTokens.named("LBRACE"), CpTokens.named("RBRACE"), false),
    )

    override fun getPairs() = pairs
    override fun isPairedBracesAllowedBeforeType(lbraceType: IElementType, contextType: IElementType?) = true
    override fun getCodeConstructStart(file: PsiFile?, openingBraceOffset: Int) = openingBraceOffset
}

