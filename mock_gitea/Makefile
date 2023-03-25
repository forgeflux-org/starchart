default: ## Debug build
	cargo build

clean: ## Clean all build artifacts and dependencies
	@-/bin/rm -rf target/
	@-/bin/rm -rf tmp/
	@cargo clean

coverage: migrate ## Generate coverage report in HTML format
	cargo tarpaulin -t 1200 --out Html \
		--skip-clean  \
		--all-features \
		--no-fail-fast

check: ## Check for syntax errors on all workspaces
	cargo check --workspace --tests --all-features

dev-env: ## Download development dependencies
	cargo fetch

doc: ## Prepare documentation
	cargo doc --no-deps --workspace --all-features

docker: ## Build docker images
	docker build \
		-t realaravinth/starchart-mock-gitea:master \
		-t realaravinth/starchart-mock-gitea:latest .

docker.run.detached: ## Run docker container in detached mode
	docker run \
	-p 8000:7000 \
	-e DATABASE_URL=/var/log/mock-gitea \
	-d \
	realaravinth/starchart-mock-gitea:latest


docker.run: ## Run docker container in detached mode
	docker run \
		-p 8000:7000 \
		-e DATABASE_URL=/var/log/mock-gitea \
		realaravinth/starchart-mock-gitea:latest


docker.publish: docker ## Build and publish docker images
	docker push realaravinth/starchart-mock-gitea:master 
	docker push realaravinth/starchart-mock-gitea:latest

lint: ## Lint codebase
	cargo fmt -v --all -- --emit files
	cargo clippy --workspace --tests --all-features

release: ## Release build
	cargo build --release

run: default ## Run debug build
	cargo run

migrate: ## run migrations
	cd migrator && cargo run
#	@-rm -rf migrator/target/

sqlx-offline-data: ## prepare sqlx offline data
	cargo sqlx prepare

test: migrate ## Run tests
	cargo test --no-fail-fast

xml-test-coverage: migrate ## Generate cobertura.xml test coverage
	cargo tarpaulin -t 1200 \
		--out XMl --skip-clean \
		--all-features --no-fail-fast

help: ## Prints help for targets with comments
	@cat $(MAKEFILE_LIST) | grep -E '^[a-zA-Z._-]+:.*?## .*$$' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
