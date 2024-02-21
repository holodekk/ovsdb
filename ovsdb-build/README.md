# ovsdb-build

Compiles an OVSDB schema into Rust models and proxies for typed database
interaction.

## Overview

In order to interact with an OVSDB instance in a type-safe manner, we need to
create strongly-typed structs that map to the tables within OVSDB.
`ovsdb-build` accepts an OVSDB schema and, for each table, generates 2 structs:

- a native struct used by the client application, and
- a proxy struct, utilized by `serde` to handle encoding/decoding messages

The conversions are primarily handled by newtype wrappers around existing Rust
types, so should be extremely efficient.

Before you can build, however, you need a schema. For instance, let's assume
you want to interact with the Open vSwitch database. The first thing you'll
need to do is obtain a copy of the schema (usually stored with a `.ovsschema`
extension). The schema file can be found in the [repository][vswitch-schema],
or obtained by connecting directly to an OVSDB instance. If you're running
Open vSwitch, then a copy of OVSDB is already running on your machine (usually
listening on a local socket). For example, on my local machine, I can connect
and download a copy of the schema with the following command:

```sh
# ovsdb-client get-schema /var/run/openvswitch/db.sock > /tmp/vswitch.ovsschema
```

Note that connecting to the OVSDB socket requires root priveleges. Before
proceeding, it is probably worth taking a look at the schema file itself, as the
OVSDB schema introduces a few concepts not present in other database systems.

[vswitch-schema]: https://github.com/openvswitch/ovs/blob/master/vswitchd/vswitch.ovsschema

## Installation

Simply add `ovsdb-build` as a dependency, either via cargo:

```sh
$ cargo add --build ovsdb-build
```

or directly to `Cargo.toml`:

```toml
[build-dependencies]
ovsdb-build = <ovsdb-version>
```

## Example

Assuming the schema downloaded earlier is in `/tmp/vswitch.ovsschema`, in `build.rs`:

```rust,no_run
fn main() -> Result<(), Box<dyn std::error::Error>> {
    ovsdb_build::configure().compile("/tmp/vswitch.ovsschema", "vswitch")?;
    Ok(())
}
```
