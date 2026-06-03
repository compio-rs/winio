#!/usr/bin/env bash

set -euo pipefail

if ! type mvn > /dev/null; then
  echo "The maven CLI, mvn, is required to run this script."
  echo "Download it from: https://maven.apache.org/download.cgi"
  exit 1
fi

version=$(grep -m 1 "version = " Cargo.toml | tr -d '"' | cut -d ' ' -f 3)

echo "Packaging v$version of the Android support component"

pushd ./android

./gradlew assembleRelease

popd

artifact_name="winio-release.aar"

artifact_path="android/winio/build/outputs/aar/$artifact_name"

mkdir -p maven
# Ensure no prior artifacts are present
git clean -dfX "./maven/"

cp ./pom-template.xml ./maven/pom.xml

# This sequence is meant to workaround the incompatibilites between macOS's sed
# command and the GNU command. Referenced from the following:
# https://stackoverflow.com/questions/5694228/sed-in-place-flag-that-works-both-on-mac-bsd-and-linux
sed -i.bak "s/\$VERSION/$version/" ./maven/pom.xml
rm ./maven/pom.xml.bak

mvn install:install-file -Dfile="$artifact_path" -Dpackaging="aar" -DpomFile="./maven/pom.xml" -DlocalRepositoryPath="./maven/"
