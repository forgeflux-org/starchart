# Architecture

Starchart is designed with maximum flexibility in mind and so it is
highly extensible. Support for new forges, federation formats and
databases can be implemented with ease and this document intends to
document how to do just that.

1. [`db-core`](../db/db-core): Contains traits(Rust-speak for
   interfaces) to implement support for new databases. Support for
   SQLite via [sqlx](https://crates.io/crates/sqlx) is implemented in
   [`db-sqlx-sqlite`](../db/db-sqlx-sqlite)

2. [`forge-core`](../forge/forge-core): Contains traits for implementing
   spidering support for a new forge type. Support for Gitea is
   implemented in [`gitea`](../forge/forge-core).

3. [`federation-core`](../federate/federate-core): Contains traits to
   implement support for new federation file formats. Support for
   [publiccodeyml](https://yml.publiccode.tools/) is implemented in
   [publiccodeyml](../federate/publiccodeyml).
