TARGET = aarch64-apple-ios-macabi
FEATURES = all,nightly,enable_log
EXAMPLE = widgets
APP_NAME = Widgets

BUILD_DIR = target/${TARGET}/debug
EXAMPLE_BIN = ${BUILD_DIR}/examples/${EXAMPLE}
APP_BUNDLE = ${BUILD_DIR}/${APP_NAME}.app
APP_EXEC = ${APP_BUNDLE}/Contents/MacOS/${EXAMPLE}
INFO_PLIST = winio-ui-ui-kit/Info.plist

.PHONY: all build bundle run debug clean

all: bundle

build:
	cargo build --target ${TARGET} --features ${FEATURES} --example ${EXAMPLE}

bundle: build ${INFO_PLIST}
	mkdir -p ${APP_BUNDLE}/Contents/MacOS
	cp ${EXAMPLE_BIN} ${APP_EXEC}
	cp ${INFO_PLIST} ${APP_BUNDLE}/Contents/
	codesign -s - -f --deep ${APP_BUNDLE}

run: bundle
	${APP_EXEC}

debug: bundle
	lldb -o run -- ${APP_EXEC}

clean:
	cargo clean
	rm -rf ${APP_BUNDLE}
