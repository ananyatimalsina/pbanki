all:
	cargo zigbuild --target armv7-unknown-linux-gnueabi.2.23 --profile dev -p pbanki --features=sdk-6-8

release:
	cargo zigbuild --target armv7-unknown-linux-gnueabi.2.23 --profile release -p pbanki --features=sdk-6-8
