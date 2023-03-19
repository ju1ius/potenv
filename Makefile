.PHONY: coverage

coverage:
	cargo llvm-cov --html --ignore-filename-regex '(tests|test_utils)\.rs'

