SHELL = /bin/bash

.PHONY: build
build: env-check avm/target/release/alan build-js-runtime anycloud/cli/target/release/anycloud
	@echo Done

.PHONY: env-check
env-check:
	bash -c "./.envcheck.sh"

.PHONY: avm-style
avm-style:
	cd avm && cargo fmt -- --check

.PHONY: avm-unit
avm-unit: compiler/alan-compile
	cd avm && cargo test

.PHONY: anycloud-style
anycloud-style:
	cd anycloud/cli && cargo fmt -- --check

.PHONY: compiler-browser-check
compiler-browser-check:
	cd compiler && yarn && yarn test

./compiler/alan-compile:
	cd compiler && yarn
	yarn add pkg
	cd compiler && ../node_modules/.bin/pkg --targets host .

./avm/target/release/alan: compiler/alan-compile
	cd avm && cargo fmt
	cd avm && cargo build --release

./anycloud/cli/target/release/anycloud: compiler/alan-compile
	cd anycloud/cli && cargo fmt
	cd anycloud/cli && cargo build --release

.PHONY: build-js-runtime
build-js-runtime:
	cd js-runtime && yarn

.PHONY: bdd
bdd: shellspec node_modules
	bash -c "./bdd/bdd.sh $(testfile)"

shellspec:
	git clone --depth 1 --branch 0.27.2 https://github.com/shellspec/shellspec

node_modules: build
	npm init -y
	yarn add ./js-runtime

.PHONY: clean
clean:
	git clean -ffdxe .vscode

.PHONY: install
install: avm/target/release/alan
	cp ./avm/target/release/alan /usr/local/bin/alan

.PHONY: uninstall
uninstall:
	rm /usr/local/bin/alan

.PHONY: version
version:
	./.version.sh $(version)

.PHONY: prerelease 
prerelease:
	./.prerelease.sh $(version)