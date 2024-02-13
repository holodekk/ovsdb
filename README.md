# ovsdb

[![Gitlab pipeline][pipeline-badge]][pipeline-url]
[![crates.io][cratesio-badge]][cratesio-url]
[![docs.rs][docsrs-badge]][docsrs-url]
[![license][license-badge]][license-url]
[![issues][issues-badge]][issues-url]

A Rust implementation of the [OVSDB][1] schema and wire format.

## What is OVSDB?

OVSDB is the database protocol behind [Open vSwitch][2] and [OVN][3], documented in
[RFC 7047][4].

## License

This project is licensed under the [MIT license](LICENSE.md).

## Author

- [Josh Williams](https://dubzland.com)

[pipeline-badge]: https://img.shields.io/gitlab/pipeline-status/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&branch=main&style=for-the-badge&logo=gitlab
[pipeline-url]: https://git.dubzland.com/holodekk/ovsdb/pipelines?scope=all&page=1&ref=main
[cratesio-badge]: https://img.shields.io/crates/v/ovsdb?style=for-the-badge&logo=rust
[cratesio-url]: https://crates.io/crates/ovsdb
[docsrs-badge]: https://img.shields.io/badge/docs.rs-ovsdb-blue?style=for-the-badge&logo=docsdotrs
[docsrs-url]: https://docs.rs/ovsdb/latest/ovsdb/
[license-badge]: https://img.shields.io/gitlab/license/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&style=for-the-badge
[license-url]: https://git.dubzland.com/holodekk/ovsdb/-/blob/main/LICENSE.md
[issues-badge]: https://img.shields.io/gitlab/issues/open/holodekk%2Fovsdb?gitlab_url=https%3A%2F%2Fgit.dubzland.com&style=for-the-badge&logo=gitlab
[issues-url]: https://git.dubzland.com/holodekk/ovsdb/-/issues
[1]: https://docs.openvswitch.org/en/latest/ref/ovsdb.7/
[2]: https://www.openvswitch.org/
[3]: https://docs.ovn.org/en/latest/contents.html
[4]: https://datatracker.ietf.org/doc/html/rfc7047
