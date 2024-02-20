//! OVSDB wire protocol implementation

mod codec;
pub use codec::{Codec, CodecError};
mod request;
pub use request::*;
mod response;
pub use response::*;

mod map;
pub use map::*;
mod message;
pub use message::Message;
pub mod method;
mod optional;
pub use optional::Optional;
mod set;
pub use set::*;
mod uuid;
pub use self::uuid::*;

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use crate::protocol;
    use serde::Deserialize;

    #[test]
    fn test_parse() {
        #[derive(Deserialize, PartialEq)]
        #[serde(rename_all = "snake_case")]
        enum TestFailMode {
            Secure,
            Standalone,
            None,
        }
        impl Default for TestFailMode {
            fn default() -> Self {
                Self::None
            }
        }
        #[derive(Clone, Deserialize, PartialEq)]
        #[serde(rename_all = "snake_case")]
        enum TestProtocols {
            OpenFlow10,
            OpenFlow11,
            OpenFlow12,
            OpenFlow13,
            OpenFlow14,
            OpenFlow15,
            None,
        }
        #[derive(Deserialize)]
        struct TestBridge {
            auto_attach: protocol::Set<protocol::Uuid>,
            controller: protocol::Set<protocol::Uuid>,
            datapath_id: protocol::Optional<String>,
            datapath_type: String,
            datapath_version: String,
            external_ids: protocol::Map<String, String>,
            fail_mode: protocol::Optional<TestFailMode>,
            flood_vlans: protocol::Set<i64>,
            flow_tables: protocol::Map<i64, protocol::Uuid>,
            ipfix: protocol::Set<protocol::Uuid>,
            mcast_snooping_enable: bool,
            mirrors: protocol::Set<protocol::Uuid>,
            name: String,
            netflow: protocol::Set<protocol::Uuid>,
            other_config: protocol::Map<String, String>,
            ports: protocol::Set<protocol::Uuid>,
            protocols: protocol::Set<TestProtocols>,
            rstp_enable: bool,
            rstp_status: protocol::Map<String, String>,
            sflow: protocol::Set<protocol::Uuid>,
            status: protocol::Map<String, String>,
            stp_enable: bool,
        }
        let data = r#"{
    "rstp_status":["map",[]],
    "_uuid":["uuid","06234b93-6b4b-4f92-be8a-342dd858617c"],
    "datapath_version":"<unknown>",
    "_version":["uuid","1ef13326-744a-4065-82ee-0998ff56dcc8"],
    "flow_tables":["map",[]],
    "protocols":["set",[]],
    "auto_attach":["set",[]],
    "mcast_snooping_enable":false,
    "flood_vlans":["set",[]],
    "stp_enable":false,
    "name":"br0",
    "sflow":["set",[]],
    "ports":["set",[
        ["uuid","67087d8a-1b61-408a-a448-a239248b9f7d"],
        ["uuid","c16f3aaa-907f-4e81-86c9-4ef845b8990e"],
        ["uuid","d05c07dd-3455-48a4-8e9e-d1f0236375b8"],
        ["uuid","d3bbf06e-e460-4162-93cf-7237bb326630"],
        ["uuid","df192fe4-a87a-45c3-b89c-a690da1d9a8d"],
        ["uuid","e5bc7326-7da8-4427-9efc-2d4b347d8add"],
        ["uuid","ef799d87-0b7c-40ad-bfab-5065970931d7"],
        ["uuid","fd190439-d354-49cd-b4cb-24f9eb7f850c"]
    ]],
    "mirrors":["set",[]],
    "netflow":["set",[]],
    "external_ids":["map",[]],
    "other_config":["map",[]],
    "datapath_type":"",
    "fail_mode":["set",[]],
    "datapath_id":"000004421af07474",
    "controller":["set",[]],
    "ipfix":["set",[]],
    "rstp_enable":false,
    "status":["map",[]]
}"#;
        let _test_bridge: TestBridge = serde_json::from_str(data).expect("TestBridge struct");
    }
}
