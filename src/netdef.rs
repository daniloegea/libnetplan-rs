use crate::libnetplan::netdef_get_id;
use crate::libnetplan::netdef_get_type;
use crate::libnetplan::NetplanNetDefinition;

pub enum NetdefType {
    None,
    Ethernet,
    Wifi,
    Modem,
    Bridge,
    Bond,
    Vlan,
    Tunnel,
    Port,
    Vrf,
    Nm,
    Dummy,
    Veth,
}

pub struct Netdef {
    pub id: String,
    pub r#type: NetdefType,
}

impl Netdef {
    pub(crate) fn from_raw_netdef(raw_netdef: *const NetplanNetDefinition) -> Self {
        let id = netdef_get_id(raw_netdef).expect("Failed to get netdef ID.");
        let netdef_type = netdef_get_type(raw_netdef);

        Netdef {
            id,
            r#type: netdef_type,
        }
    }
}
