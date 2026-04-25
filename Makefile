release-arm64:
	./release-arm64.sh

serve:
	cd frontend && npm run build && cd .. && cargo run -- serve

serve-dev:
	cargo run -- serve

publish:
	cargo publish --registry github --token $(CARGO_REGISTRY_TOKEN)

clippy:
	cargo clippy --all-targets --all-features

fix:
	cargo fmt
	cargo clippy --all-targets --all-features --fix --allow-dirty
	cargo fix --allow-dirty

check:
	cargo fmt -- --check

test:
	cargo test
