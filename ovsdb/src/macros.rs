#[macro_export]
macro_rules! include_schema {
    ($schema: tt) => {
        include!(concat!(
            env!("OUT_DIR"),
            concat!("/", $schema, "/", "mod.rs")
        ));
    };
}
