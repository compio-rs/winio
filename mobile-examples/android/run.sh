#!/bin/bash

adb logcat -c && \
cargo apk2 run --example hello
