plugins {
    id("com.android.application")
}

android {
    namespace = "com.arrowsk.astronomyobserver"
    compileSdk = 36
    buildToolsVersion = "35.0.0"

    defaultConfig {
        applicationId = "com.arrowsk.astronomyobserver"
        minSdk = 28
        targetSdk = 36
        versionCode = 30200
        versionName = "0.3.2"

        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDir("../generated/jniLibs")
            assets.srcDirs("src/main/assets", "../generated/assets", "../../astronomy_observer/data")
        }
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
        }
    }

    packaging {
        jniLibs {
            useLegacyPackaging = false
        }
        resources {
            excludes += setOf("META-INF/DEPENDENCIES", "META-INF/LICENSE*", "META-INF/NOTICE*")
        }
    }

    lint {
        abortOnError = true
        warningsAsErrors = false
        checkReleaseBuilds = true
    }
}
