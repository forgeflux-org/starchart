# Hacking

Instructions WIP. Kindly give feedback :)

## Development dependencies

1. [pkg-config](https://packages.debian.org/bullseye/pkg-config)
2. [GNU make](https://packages.debian.org/bullseye/make)
3. [libssl-dev](https://packages.debian.org/bullseye/libssl-dev)
4. Rust(see [installation instructions](#install-rust))
5. Docker-compose

### Install Rust

Install Rust using [rustup](https://rustup.rs/).

`rustup` is the official Rust installation tool. It enables installation
of multiple versions of `rustc` for different architectures across
multiple release channels(stable, nightly, etc.).

Rust undergoes [six-week release
cycles](https://doc.rust-lang.org/book/appendix-05-editions.html#appendix-e---editions)
and some of the dependencies that are used in this program have often
relied on cutting edge features of the Rust compiler. OS Distribution
packaging teams don't often track the latest releases. For this reason,
we encourage managing your Rust installation with `rustup`.

**`rustup` is the officially supported Rust installation method of this
project.**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Implementing Support for $FORGE

> In future, Starchart will be modified to talk forge federation
> ActivityPub protocol(general term, not referring to
> [forgefed](https://forgefed.peers.community/)), so implementing support
> for $FORGE would mean implementing that protocol for $FORGE.

**TODO**

### Testing

**2022-04-13:** Support for [Gitea](https://gitea.io) is WIP and because
Gitea is Free Software and light-weight to run within CI/CD environment,
we are able to run a Gitea instate and run tests against it. See
[docker-compose-dev-deps.yml](../docker-compose-dev-deps.yml).

## Implementing Support for $DATABASE

> Thank you for your interest in adding support for a new database. Please let us know about your effort
> so that we can link to it on this repository :)

Starchart defines all database operations in [`db-core`](../db/db-core])
local crate. Implementing `SCDatabase` from the same crate will add
support for your database.

### Testing

Tests are generic over all database support implementations, so tests
are implemented as part of the core package at
[db-core/tests.rs](../db/db-core/src/tests.rs) and re-exported for use
within tests.

Please see
[SQLite tests implementation](../db/db-sqlx-sqlite/src/tests.rs) for
inspiration.
