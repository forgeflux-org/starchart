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

define cache_bust ## run cache_busting program
	cd utils/cache-bust && cargo run
endef

define test_workspaces
	$(call test_databases)
	$(call test_forges)
	$(call test_federation)
	DATABASE_URL=${SQLITE_DATABASE_URL}\
		cargo test --no-fail-fast
endef

default: ## Debug build
	$(call cache_bust)
	cargo build

env.serve:
	STARCHART__LOG=info \
	STARCHART__SOURCE_CODE="https://github.com/forgeflux-org/starchart" \
	STARCHART__ALLOW_NEW_INDEX=true \
	STARCHART__ADMIN_EMAIL=realaravinth@batsense.net \
	STARCHART__SERVER__IP=0.0.0.0 \
	STARCHART__SERVER__PORT=7000 \
	STARCHART__SERVER__DOMAIN=localhost \
	STARCHART__SERVER__PROXY_HAS_TLS=false \
	STARCHART__SERVER__COOKIE_SECRET=7514316e58bfdb2eb2d71bf4af40827a \
	STARCHART__DATABASE__POOL=5 STARCHART__DATABASE__TYPE=sqlite \
	STARCHART__CRAWLER__TTL=3600 \
	STARCHART__CRAWLER__WAIT_BEFORE_NEXT_API_CALL=2 \
	STARCHART__CRAWLER__CLIENT_TIMEOUT=60 \
	STARCHART__CRAWLER__ITEMS_PER_API_CALL=20 \
	STARCHART__INTRODUCER__PUBLIC_URL="http://localhost:7000" \
	STARCHART__INTRODUCER__NODES=http://localhost:7001,http://localhost:7002 \
	STARCHART__REPOSITORY__ROOT=/tmp/starchart.forgeflux.org \
	cargo run

cache-bust: ## Run cache buster on static assets
	$(call cache_bust)

clean: ## Clean all build artifacts and dependencies
	@-/bin/rm -rf target/
	@-/bin/rm -rf database/migrator/target/
	@-/bin/rm -rf database/*/target/
	@-/bin/rm -rf database/*/tmp/
	@cargo clean

coverage: migrate ## Generate coverage report in HTML format
	$(call launch_test_env)
	$(call cache_bust)
	cargo tarpaulin -t 1200 --out Html --skip-clean  --all-features --no-fail-fast --workspace=db/db-sqlx-sqlite,forge/gitea,federate/publiccodeyml,.

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
	cd utils/cache-bust && cargo check --tests --all-features
	cd api_routes && cargo check --tests --all-features

dev-env: ## Download development dependencies
	$(call launch_test_env)
	cargo fetch

doc: ## Prepare documentation
	cargo doc --no-deps --workspace --all-features

docker: ## Build docker images
	docker build -t forgedfed/starchart:master -t forgedfed/starchart:latest .

docker-publish: docker ## Build and publish docker images
	docker push forgedfed/starchart:master 
	docker push forgedfed/starchart:latest

lint: ## Lint codebase
	cargo fmt -v --all -- --emit files
	cargo clippy --workspace --tests --all-features

release: ## Release build
	$(call cache_bust)
	cargo build --release

run: default ## Run debug build
	cargo run

migrate: ## run migrations
	@-rm -rf db/db-sqlx-sqlite/tmp && mkdir db/db-sqlx-sqlite/tmp
	@-rm -rf db/migrator/target/
	cd db/migrator && cargo run
#	echo TODO: add migrations

sqlx-offline-data: ## prepare sqlx offline data
	cd db/db-sqlx-sqlite/ \
		&& DATABASE_URL=${SQLITE_DATABASE_URL} cargo sqlx prepare
#	cargo sqlx prepare  --database-url=${POSTGRES_DATABASE_URL} -- --bin starchart \
		--all-features
test: migrate ## Run tests
	$(call launch_test_env)
	$(call cache_bust)
	$(call test_workspaces)

#	cd database/db-sqlx-postgres &&\
#		DATABASE_URL=${POSTGRES_DATABASE_URL}\
#		cargo test --no-fail-fast

xml-test-coverage: migrate ## Generate cobertura.xml test coverage
	$(call launch_test_env)
	$(call cache_bust)
	cargo tarpaulin -t 1200 --out XMl --skip-clean  --all-features --no-fail-fast --workspace=db/db-sqlx-sqlite,forge/gitea,federate/publiccodeyml,.

network.up: ## Deploy Gitea network
	docker-compose -f ./foo.yml up --detach

network.down.rm: ## Remove gitea network, removing containers
	docker-compose -f ./foo.yml down --remove-orphans -v

network.down: ## Remote Gitea network
	docker-compose -f ./foo.yml down

network.docker-config.init: ## Generate docker-compose for gitea network
	rm -rf ./foo.yml || true
	./gitea.sh > foo.yml

network.logs: ## network logs
	docker-compose -f ./foo.yml logs -f

network.init: ## Initialize gitea network
	@ . ./venv/bin/activate && python ./scripts/gitea.py

help: ## Prints help for targets with comments
	@cat $(MAKEFILE_LIST) | grep -E '^[a-zA-Z._-]+:.*?## .*$$' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
