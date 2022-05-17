define launch_test_env
	docker-compose -f docker-compose-dev-deps.yml up --detach
	python ./scripts/gitea.py
endef

define test_databases
	cd db/db-core &&\
		cargo test --no-fail-fast
	cd db/db-sqlx-sqlite &&\
		DATABASE_URL=${SQLITE_DATABASE_URL}\
		cargo test --no-fail-fast
endef

define test_forges
	cd forge/forge-core && \
		cargo test --no-fail-fast
	cd forge/gitea && \
		cargo test --no-fail-fast
endef

define test_federation
	cd federate/federate-core && \
		cargo test --no-fail-fast
	cd federate/publiccodeyml && \
		cargo test --no-fail-fast
endef

define test_workspaces
	$(call test_databases)
	$(call test_forges)
	$(call test_federation)
	cargo test --no-fail-fast
endef

default: ## Debug build
	cargo build

clean: ## Clean all build artifacts and dependencies
	@-/bin/rm -rf target/
	@-/bin/rm -rf database/migrator/target/
	@-/bin/rm -rf database/*/target/
	@-/bin/rm -rf database/*/tmp/
	@cargo clean

coverage: migrate ## Generate coverage report in HTML format
	cargo tarpaulin -t 1200 --out Html --skip-clean  --all-features --no-fail-fast #--workspace=database/db-sqlx-postgres,database/db-sqlx-sqlite,.

check: ## Check for syntax errors on all workspaces
	cargo check --workspace --tests --all-features
	cd db/db-sqlx-sqlite &&\
		DATABASE_URL=${SQLITE_DATABASE_URL}\
		cargo check
	cd db/db-core/ && cargo check
	cd db/migrator && cargo check --tests --all-features
	cd forge/forge-core && cargo check --tests --all-features
	cd forge/gitea && cargo check --tests --all-features
	cd federate/federate-core && cargo check --tests --all-features
	cd federate/publiccodeyml && cargo check --tests --all-features

dev-env: ## Download development dependencies
	$(call launch_test_env)
	cargo fetch

doc: ## Prepare documentation
	cargo doc --no-deps --workspace --all-features

#docker: ## Build docker images
#	docker build -t realaravinth/gitpad:master -t realaravinth/gitpad:latest .
#
#docker-publish: docker ## Build and publish docker images
#	docker push realaravinth/gitpad:master 
#	docker push realaravinth/gitpad:latest

lint: ## Lint codebase
	cargo fmt -v --all -- --emit files
	cargo clippy --workspace --tests --all-features

release: ## Release build
	cargo build --release

run: default ## Run debug build
	cargo run

migrate: ## run migrations
	@-rm -rf db/db-sqlx-sqlite/tmp && mkdir db/db-sqlx-sqlite/tmp
	cd db/migrator && cargo run
#	echo TODO: add migrations

sqlx-offline-data: ## prepare sqlx offline data
	cd db/db-sqlx-sqlite/ \
		&& DATABASE_URL=${SQLITE_DATABASE_URL} cargo sqlx prepare
#	cargo sqlx prepare  --database-url=${POSTGRES_DATABASE_URL} -- --bin starchart \
		--all-features
test: migrate ## Run tests
	$(call launch_test_env)
	$(call test_workspaces)

#	cd database/db-sqlx-postgres &&\
#		DATABASE_URL=${POSTGRES_DATABASE_URL}\
#		cargo test --no-fail-fast

xml-test-coverage: migrate ## Generate cobertura.xml test coverage
	cargo tarpaulin -t 1200 --out Xml --skip-clean --all-features --no-fail-fast #--workspace=database/db-sqlx-postgres,database/db-sqlx-sqlite,.

help: ## Prints help for targets with comments
	@cat $(MAKEFILE_LIST) | grep -E '^[a-zA-Z_-]+:.*?## .*$$' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
