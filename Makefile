.PHONY: docker-build
docker-build:
	docker build . -t rust-linux-worker:latest

.PHONY: docker-run
docker-run:
	docker run \
		--privileged \
		-p 50051:50051 \
		rust-linux-worker:latest

.PHONY: fmt
fmt:
	cargo +nightly fmt
