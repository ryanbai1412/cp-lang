package dev.cplang.ide

import com.intellij.extapi.psi.PsiFileBase
import com.intellij.lang.Language
import com.intellij.openapi.fileTypes.LanguageFileType
import com.intellij.openapi.util.IconLoader
import com.intellij.psi.FileViewProvider
import javax.swing.Icon

object CpLanguage : Language("CP") {
    private fun readResolve(): Any = CpLanguage
    override fun getDisplayName() = "CP Lang"
}

object CpIcons {
    @JvmField
    val FILE: Icon = IconLoader.getIcon("/icons/cp.svg", CpIcons::class.java)
}

object CpFileType : LanguageFileType(CpLanguage) {
    override fun getName() = "CP Lang"
    override fun getDescription() = "CP Lang source"
    override fun getDefaultExtension() = "cp"
    override fun getIcon() = CpIcons.FILE
}

class CpFile(viewProvider: FileViewProvider) : PsiFileBase(viewProvider, CpLanguage) {
    override fun getFileType() = CpFileType
    override fun toString() = "CP Lang file"
}
