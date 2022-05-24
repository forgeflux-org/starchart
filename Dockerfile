FROM rust:latest as rust
WORKDIR /src
#RUN mkdir /src/scripts
#COPY scripts/docker-cache.sh /src/scripts/docker-cache.sh
#RUN ./scripts/docker-cache.sh
#
#COPY Cargo.* /src/
#
#COPY db/db-core/Cargo.* /src/db/db-core/
#
#COPY db/db-sqlx-sqlite/Cargo.* /src/db/db-sqlx-sqlite/
#
#COPY db/migrator/Cargo.* /src/db/migrator/
#
#COPY forge/forge-core/Cargo.* /src/forge/forge-core/
#
#COPY forge/gitea/Cargo.* /src/forge/gitea/
#COPY utils/cache-bust/Cargo.* /src/utils/cache-bust/
#COPY federate/federate-core/Cargo.* /src/federate/federate-core/
#COPY federate/publiccodeyml/Cargo.* /src/federate/publiccodeyml/
#RUN find /src/
#RUN sed -i '/.*build.rs.*/d' Cargo.toml
#
#COPY db/db-sqlx-sqlite/migrations/ /src/db/db-sqlx-sqlite/migrations/
#COPY db/db-sqlx-sqlite/sqlx-data.json /src/db/db-sqlx-sqlite/sqlx-data.json
#COPY Makefile /src/
#
#RUN cargo --version
#RUN make release
COPY . /src/
RUN make release

FROM debian:bullseye-slim
LABEL org.opencontainers.image.source https://github.com/forgeflux-org/starchart
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=rust /src/target/release/starchart /usr/local/bin/
COPY --from=rust /src/config/default.toml /etc/starchart/config.toml
COPY scripts/entrypoint.sh /usr/local/bin
RUN chmod +x /usr/local/bin/entrypoint.sh
CMD [ "/usr/local/bin/entrypoint.sh" ]
