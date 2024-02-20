/// Include generated schema items.
///
/// Native structs can be generated for a given schema using `ovsdb-build`.  This macro allows
/// those structs to be used in normal rust code.
///
/// ```rust,ignore
/// mod vswitch {
///   ovsdb::include_schema!("vswitch")
/// }
/// ```
///
/// The schema name must match the name used in the `ovsdb-build` process.
#[macro_export]
macro_rules! include_schema {
    ($schema: tt) => {
        include!(concat!(
            env!("OUT_DIR"),
            concat!("/", $schema, "/", "mod.rs")
        ));
    };
}
