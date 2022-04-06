all: release bench

release:
	cargo rustc --release --lib -- -C target-feature=+popcnt -C target-cpu=native
	cargo rustc --release --bin crabby -- -C target-feature=+popcnt -C target-cpu=native

# To run benchmarks with fully optimized code
bench:
	cargo bench --bench testing

clean:
	rm -rf target
