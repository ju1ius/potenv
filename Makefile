.PHONY: coverage

coverage:
	cargo llvm-cov --html --open --ignore-filename-regex 'test(s|_utils)\.rs'

docs:
	cargo doc --no-deps --open
