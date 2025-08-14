#!/bin/bash
set -o nounset -o pipefail -o errexit
binary_name=$1
app_name="$binary_name.app"

# make the AppIcon.icns file from the iconset
iconutil -c icns appicon.iconset

export MACOSX_DEPLOYMENT_TARGET="14"
cargo build --release --target aarch64-apple-darwin --no-default-features
mkdir -p $app_name/Contents/MacOS
mkdir -p $app_name/Contents/Resources
cp AppIcon.icns $app_name/Contents/Resources/
cp target/aarch64-apple-darwin/release/$binary_name $app_name/Contents/MacOS/
cp -r assets "$app_name/Contents/MacOS/"


cat <<EOF > "$app_name/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$binary_name</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon.icns</string>
</dict>
</plist>
EOF

hdiutil create -fs HFS+ -volname "$binary_name-macOS-apple-silicon" -srcfolder $app_name $binary_name-macOS-apple-silicon.dmg

# Note for anyone who may want to use the app. After downloading a copy
# of the game, install it to Applications. Then run:
#   sudo xattr -rd com.apple.quarantine /Applications/bevy-dino.app
