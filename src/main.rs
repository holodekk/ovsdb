use ovsdb_rust::OVSDBClient;
use std::path::Path;

fn main() {
    let mut client = OVSDBClient::connect_unix(Path::new("/var/run/openvswitch/db.sock")).unwrap();
    match client.list_bridges().unwrap() {
        Some(bridges) => {
            println!("Got some bridges:");
            for bridge in &bridges {
                println!("{:?}", bridge);
            }
        }
        None => {
            println!("Got nothing")
        }
    }

    client.disconnect().unwrap();
}
