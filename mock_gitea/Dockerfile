FROM rust:latest as planner
RUN cargo install cargo-chef
WORKDIR /src
COPY . /src/
RUN cargo chef prepare --recipe-path recipe.json


FROM rust:latest as cacher
WORKDIR /src/
RUN cargo install cargo-chef
COPY --from=planner /src/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json


FROM rust:latest as builder
WORKDIR /src/
COPY . .
COPY --from=cacher /src/target target
RUN make release

FROM debian:bullseye-slim
LABEL org.opencontainers.image.source https://github.com/forgeflux-org/starchart
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /src/target/release/mock_gitea /usr/local/bin/
COPY --from=builder /src/config/default.toml /etc/mock_gitea/config.toml
COPY ./data/unique.txt /var/lib/unique.txt
RUN find /var/lib
COPY scripts/entrypoint.sh /usr/local/bin
RUN chmod +x /usr/local/bin/entrypoint.sh
CMD [ "/usr/local/bin/entrypoint.sh" ]
