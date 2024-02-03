use std::path::Path;

use ovsdb::Client;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut client = Client::connect_unix(Path::new("/var/run/openvswitch/db.sock"))
        .await
        .unwrap();

    let res: Vec<String> = client.echo(vec![]).await.unwrap();
    println!("Got an echo response: {:#?}", res);

    let s: ovsdb::schema::Schema = client.get_schema("Open_vSwitch").await.unwrap();
    println!("Got a schema: {:#?}", s);

    Ok(())
}
