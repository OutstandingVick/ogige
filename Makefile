.PHONY: test verify install fixtures review

test:
	cargo test --locked

fixtures:
	cargo run --locked --example generate_sdk_fixtures
	git diff --exit-code -- fixtures/sdk

verify:
	./showcase/telegram-firewall/bin/verify

install:
	./showcase/telegram-firewall/bin/install

review:
	@test -n "$(REQUEST)" || (echo "usage: make review REQUEST=/absolute/path/request.json" >&2; exit 2)
	./showcase/telegram-firewall/bin/ogige-review "$(REQUEST)"
