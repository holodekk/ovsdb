# ovsdb

[![Gitlab pipeline][pipeline-badge]][pipeline-url]
[![crates.io][cratesio-badge]][cratesio-url]
[![docs.rs][docsrs-badge]][docsrs-url]
[![license][license-badge]][license-url]
[![issues][issues-badge]][issues-url]

A Rust implementation of the [`OVSDB`] schema and wire format.

## What is OVSDB?

OVSDB is the database protocol behind [Open vSwitch][vswitch] and [OVN], documented in
[RFC 7047][OVSDB-RFC]. If you don't know what either of those technologies are, you're
probably in the wrong place.

At the core of OVSDB are the schema and protocol. Together they describe both
the structure of data in the database, and the manner in which that data can be
manipulated. Both are described in detail in the RFC mentioned above (which
you really should read).

## Overview

[`ovsdb`] provides a Rust interface to an OVSDB server. It utilizes [`serde`]
for protocol processing and [`tokio`] for asynchronous io. Its features
include:

- automated generation of Rust models via [`ovsdb-build`]
- strongly typed interfaces to OVSDB data structures
- automatic conversion to/from OVSDB protocol types

## Project Layout

- [`ovsdb`](https://github.com/holodekk/ovsdb/tree/master/ovsdb): Protocol/schema and client implementations
- [`ovsdb-build`](https://github.com/holodekk/ovsdb/tree/master/ovsdb-build): Model generation
- [`examples`](https://github.com/holodekk/ovsdb/tree/master/examples): Sample OVSDB interactions

## Contributing

As this crate is heavily WIP, contributions are extremely welcome. Check out
the [contributing guide][guide] to get involved.

[guide]: CONTRIBUTING.md

## License

This project is licensed under the [MIT license](LICENSE.md).

## Author

- [Josh Williams](https://dubzland.com)

[pipeline-badge]: https://img.shields.io/gitlab/pipeline-status/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&branch=main&style=flat-square&logo=gitlab
[pipeline-url]: https://git.dubzland.com/holodekk/ovsdb/pipelines?scope=all&page=1&ref=main
[cratesio-badge]: https://img.shields.io/crates/v/ovsdb?style=flat-square&logo=rust
[cratesio-url]: https://crates.io/crates/ovsdb
[docsrs-badge]: https://img.shields.io/badge/docs.rs-ovsdb-blue?style=flat-square&logo=docsdotrs
[docsrs-url]: https://docs.rs/ovsdb/latest/ovsdb/
[license-badge]: https://img.shields.io/gitlab/license/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&style=flat-square
[license-url]: https://git.dubzland.com/holodekk/ovsdb/-/blob/main/LICENSE.md
[issues-badge]: https://img.shields.io/gitlab/issues/open/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&style=flat-square&logo=gitlab
[issues-url]: https://git.dubzland.com/holodekk/ovsdb/-/issues
[`OVSDB`]: https://docs.openvswitch.org/en/latest/ref/ovsdb.7/
[vswitch]: https://www.openvswitch.org/
[OVN]: https://docs.ovn.org/en/latest/contents.html
[OVSDB-RFC]: https://datatracker.ietf.org/doc/html/rfc7047
[`ovsdb-build`]: https://docs.rs/ovsdb-build
[`serde`]: https://docs.rs/serde
[`tokio`]: https://docs.rs/tokio
