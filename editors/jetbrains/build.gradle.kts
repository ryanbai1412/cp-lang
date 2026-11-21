import org.jetbrains.intellij.platform.gradle.TestFrameworkType

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.0.21"
    id("org.jetbrains.intellij.platform") version "2.2.1"
}

group = "dev.cplang"
version = providers.gradleProperty("pluginVersion").get()

kotlin {
    jvmToolchain(21)
}

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        // The LSP API ships in the commercial IDEs (IntelliJ IDEA Ultimate, CLion,
        // PyCharm Professional, ...), so the plugin builds against one of them.
        intellijIdeaUltimate(providers.gradleProperty("platformVersion"))
        testFramework(TestFrameworkType.Platform)
    }
    testImplementation("junit:junit:4.13.2")
}

intellijPlatform {
    pluginConfiguration {
        version = providers.gradleProperty("pluginVersion")
        ideaVersion {
            sinceBuild = "243"
            untilBuild = provider { null }
        }
    }
    buildSearchableOptions = false
    // No GUI forms or @NotNull to instrument, and its compiler is one more download.
    instrumentCode = false
}

val repo: File = rootDir.parentFile.parentFile

tasks {
    test {
        systemProperty("cpl.repo", repo.absolutePath)
        systemProperty("cpl.bin", System.getenv("CPL") ?: repo.resolve("target/debug/cpl").absolutePath)
    }
}
