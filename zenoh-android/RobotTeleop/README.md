# <img src="android-robot.png" alt="Android" height="50"> Zenoh Android Demo App

An Android application using the [Zenoh-Kotlin bindings](https://github.com/eclipse-zenoh/zenoh-kotlin).

The application features robotic teleoperation via Zenoh on your android device

## Requirements

To install this application, it's required:

- The Android SDK 30+ (so this app is compatible with Android 11)
- Gradle 8
- Because it relies on the Zenoh-Kotlin library which contains the Zenoh-JNI native library, the
  example works on mobile phones with the following CPU architectures:
  - ARM
  - ARM64
  - X86
  - X86_64

## Configuration

This Android app imports the zenoh kotlin library from [Github packages](https://github.com/eclipse-zenoh/zenoh-kotlin/packages/1968034). Eventhough it's a public
package, Github requires us to specify an user and a token to download the package. Therefore it's
necessary that you specify on the `gradle.properties` file your github user and github access token as
`user` and `token` respectively

```
user=<github_account>
token=<github_token>
```

That will enable the github packages repository (see [settings.gradle.kts](/settings.gradle.kts)).

## Installation

### Android Studio

The simplest way is to install [Android Studio](https://developer.android.com/studio#command-tools), which makes the installation of the Android SDK straightforward, as well as the Gradle installation. Then we can simply load and build the project.

### Android Command Line Tools

In case we want to install the application without Android Studio then:

1. Install the Android Command Line Tools ([link](https://developer.android.com/studio#command-tools))
2. Export the `ANDROID_HOME` environment variable and update the `PATH` variable as well:
   ```bash
   $ export ANDROID_HOME=/path/to/android-command-line-tools
   $ export PATH=$PATH:$ANDROID_HOME/bin
   ```
3. Install Gradle ([link](https://gradle.org/))
4. On this project's directory, run
   ```bash
   $ gradle wrapper
   $ ./gradlew assembleDebug # or assembleRelease
   ```
5. Install the app:
   ```bash
   $ adb install -r app/build/outputs/apk/debug/app-debug.apk
   ```
6. Run the app on the phone

## Logs

To enable the zenoh logs (visible on LogCat) add the following entry to the `gradle.properties` file:

```
zenoh.logger=debug
```
