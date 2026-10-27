# Working preferences

- Do not build Windows or Linux app binaries, installers, AppImages, Debian packages or distribution bundles by default. Build these only when the user explicitly requests a build.
- For ordinary implementation work, run relevant automated code checks and commit the completed batch. Leave app builds, native app testing and physical networking acceptance to the user unless explicitly requested.
- Do not start WSL/Linux build or packaging workflows as an automatic follow-up to a code change.
