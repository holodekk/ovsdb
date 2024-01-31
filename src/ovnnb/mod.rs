// use std::cell::{Ref, RefCell};
// use std::path::Path;

// use serde::{Deserialize, Serialize};
// // use uuid::Uuid;

// use crate::ovsdb;

// #[derive(Debug, Deserialize, Serialize)]
// pub struct LogicalSwitch {
//     pub _uuid: ovsdb::Atom,
//     pub name: String,
// }

// impl ovsdb::Entity for LogicalSwitch {
//     fn table_name() -> &'static str {
//         "Logical_Switch"
//     }

//     //     fn load_table<'a>(data: &'a [u8]) -> Result<Option<Vec<Self>>, ovs::Error> {
//     //         let res: ovs::ListResponse<LogicalSwitch> = serde_json::from_slice(data).unwrap();
//     //         match res.result.into_iter().next() {
//     //             Some(result) => Ok(Some(result.rows)),
//     //             None => Ok(None),
//     //         }
//     //     }
// }

// pub struct Client<T>
// where
//     T: ovsdb::Connection,
// {
//     conn: RefCell<T>,
// }

// impl<T> ovsdb::Client<T> for Client<T>
// where
//     T: ovsdb::Connection,
// {
//     fn disconnect(&self) -> Result<(), ovsdb::Error> {
//         self.conn.borrow_mut().disconnect()
//     }

//     fn conn(&self) -> Ref<T> {
//         self.conn.borrow()
//     }

//     fn database(&self) -> &str {
//         "OVN_Northbound"
//     }
// }

// impl Client<ovsdb::UnixConnection> {
//     pub fn connect_unix() -> Result<Self, ovsdb::Error> {
//         let conn = ovsdb::UnixConnection::connect(Path::new("/var/run/ovn/ovnnb_db.sock"))?;
//         Ok(Self {
//             conn: RefCell::new(conn),
//         })
//     }
// }
