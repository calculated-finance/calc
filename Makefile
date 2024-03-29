.PHONY: deploy-local e2etest e2etest-full lint optimize publish-packages publish-contracts schema release

TEST_ADDRS ?= $(shell jq -r '.[].address' ./e2e/configs/test_accounts.json | tr '\n' ' ')
GAS_LIMIT ?= "75000000"

deploy-local:
	sudo docker kill archway || true
	sudo docker volume rm -f archway_data
	sudo docker build e2e/archway -t archway-local
	sudo docker run --rm -d --name archway \
		-e DENOM=aarch \
		-e CHAINID=testing \
		-e GAS_LIMIT=$(GAS_LIMIT) \
		-p 1317:1317 \
		-p 26656:26656 \
		-p 26657:26657 \
		-p 9090:9090 \
		--mount type=volume,source=archway_data,target=/root \
		archway-local $(TEST_ADDRS)
		

e2etest:
	RUST_LOG=info CONFIG=configs/cosm-orc.yaml cargo integration-test $(TEST_NAME)

e2etest-full: deploy-local optimize e2etest

lint:
	cargo clippy --all-targets -- -D warnings

optimize:
	# NOTE: On a cache miss, the dockerized workspace-optimizer container
	# is creating these dirs with permissions we cannot use in CI.
	# So, we need to ensure these dirs are created before calling optimize.sh:
	mkdir -p artifacts target
	sh scripts/optimize.sh


schema:
	sh scripts/schema.sh $(VERSION)

release:
	sh scripts/release.sh $(VERSION)

upload:
	sh scripts/upload.sh