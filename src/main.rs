use std::path::Path;

use ovsdb::vswitch;
use ovsdb::{
    connect_unix,
    request::{GetSchemaParams, Method},
    response::Response,
    schema::Schema,
    Client, Connection,
};

fn get_schema<T>(client: &Client<T>)
where
    T: Connection,
{
    let res: Response<Schema> = client
        .execute(&Method::GetSchema(GetSchemaParams::new(
            vswitch::DATABASE_NAME,
        )))
        .unwrap();
    print!("schema: {:#?}", res.result);
}

fn main() {
    let client = connect_unix(Path::new("/var/run/openvswitch/db.sock")).unwrap();

    get_schema(&client);
}
