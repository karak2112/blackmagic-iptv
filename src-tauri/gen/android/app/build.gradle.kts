import java.util.Properties

fun loadKeyValueFile(path: java.io.File): Map<String, String> =
    path.readLines()
        .map { it.trim() }
        .filter { it.isNotEmpty() && !it.startsWith("#") }
        .mapNotNull { line ->
            val idx = line.indexOf('=')
            if (idx < 1) null else line.substring(0, idx).trim() to line.substring(idx + 1).trim()
        }
        .toMap()

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

val appVersionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
val apkBaseName = "blackmagic-iptv-$appVersionName"

android {
    compileSdk = 36
    namespace = "com.blackmagicsoftware.iptv"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "true"
        applicationId = "com.blackmagicsoftware.iptv"
        minSdk = 24
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = appVersionName
    }
    signingConfigs {
        create("release") {
            val keystorePropertiesFile = rootProject.file("keystore.properties")
            if (keystorePropertiesFile.exists()) {
                val keystoreProperties = loadKeyValueFile(keystorePropertiesFile)
                keyAlias = keystoreProperties.getValue("keyAlias")
                keyPassword = keystoreProperties.getValue("password")
                storeFile = file(keystoreProperties.getValue("storeFile"))
                storePassword = keystoreProperties.getValue("password")
            }
        }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            if (rootProject.file("keystore.properties").exists()) {
                signingConfig = signingConfigs.getByName("release")
            }
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

afterEvaluate {
    tasks.matching {
        it.name.startsWith("assemble") &&
            !it.name.contains("AndroidTest") &&
            !it.name.contains("UnitTest")
    }.configureEach {
        doLast {
            val apkRoot = layout.buildDirectory.dir("outputs/apk").get().asFile
            if (!apkRoot.exists()) return@doLast

            apkRoot.walkTopDown()
                .filter { it.isFile && it.extension == "apk" }
                .forEach { apk ->
                    val buildType = apk.parentFile.name
                    val abiCandidate = apk.parentFile.parentFile?.name
                    val abiPart = when {
                        abiCandidate == null || abiCandidate == "apk" -> "universal"
                        abiCandidate == "release" || abiCandidate == "debug" -> "universal"
                        else -> abiCandidate
                    }
                    val newName = "$apkBaseName-$abiPart-$buildType.apk"
                    val dest = File(apk.parentFile, newName)
                    if (apk.name != newName) {
                        apk.renameTo(dest)
                    }
                }
        }
    }
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.lifecycle:lifecycle-process:2.10.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")