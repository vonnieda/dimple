#!/bin/bash

APP="../target/release/bundle/osx/Dimple.app"
BIN="$APP/Contents/MacOS/dimple_ui_slint"
ZIP="$APP/../Dimple.app.zip"
IDENT="KZ3MZ5JYNR"
USERNAME="jason@vonnieda.org"
PASSWORD=$NOTARIZE_PASSWORD

cargo bundle --release
codesign --timestamp --verify -vvv --deep --options=runtime --sign $IDENT $APP
zip -r $ZIP $APP
xcrun notarytool submit --apple-id $USERNAME --team-id $IDENT --password $PASSWORD --wait $ZIP
xcrun stapler staple $APP
sudo rm -rf /Users/jason/Applications/Dimple.app
sudo cp -r $APP /Users/jason/Desktop
sudo cp -r $APP /Users/jason/Applications

