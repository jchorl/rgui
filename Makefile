run:
	docker run -it --rm \
		-v $(PWD):/rgui \
		-w /rgui \
		-u 1000:1000 \
		jchorl/rgui \
		cargo run -- $(rgargs)

rust:
	docker run -it --rm \
		-v $(PWD):/rgui \
		-w /rgui \
		-u 1000:1000 \
		jchorl/rgui \
		bash

release:
	docker run -it --rm \
		-v $(PWD):/rgui \
		-w /rgui \
		-u 1000:1000 \
		rust:1.42 \
		cargo build --release

image:
	docker build -t jchorl/rgui .
