use ovsdb_rust::ovnnb;
use ovsdb_rust::ovs::Client;
use ovsdb_rust::ovsdb;

fn list_bridges() {
    let client = ovsdb::Client::connect_unix().unwrap();
    match client.list::<ovsdb::Bridge>().unwrap() {
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

fn list_switches() {
    let client = ovnnb::Client::connect_unix().unwrap();
    match client.list::<ovnnb::LogicalSwitch>().unwrap() {
        Some(switches) => {
            println!("Got some switches:");
            for switch in &switches {
                println!("{:?}", switch);
            }
        }
        None => {
            println!("Got nothing")
        }
    }

    client.disconnect().unwrap();
}

fn main() {
    list_bridges();
    list_switches();
}
