.PHONY: test dev

test: dev
	.venv/bin/python examples/basic/train.py

dev: .venv
	cargo build -p modelfox_python
	cp ../../target/debug/libmodelfox_python.so modelfox/modelfox_python.so
	.venv/bin/pip install -e .

.venv:
	virtualenv .venv
