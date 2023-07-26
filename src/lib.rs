pub mod futuapi;
pub use futuapi::{FutuApi, UserInfo};
pub mod common;
pub use common::{SecruityGroupData, SecurityGroupType, SecurityStaticBasic, SecurityStaticInfo};
pub mod protocol;
pub mod qot;
pub mod trd;

// pub mod common {
//   include!(concat!(env!("OUT_DIR"), "/common.rs"));
// }

// pub mod pb {
//   pub mod common {
//     include!(concat!("../target/debug/build/pb", "/common.rs"));
//   }
//   pub mod protocol {
//     include!(concat!("../target/debug/build/pb", "/protocol.rs"));
//   }
//   pub mod init_connect {
//     include!(concat!("../target/debug/build/pb", "/init_connect.rs"));
//   }
//   pub mod keep_alive {
//     include!(concat!("../target/debug/build/pb", "/keep_alive.rs"));
//   }
//   pub mod userinfo {
//     include!(concat!("../target/debug/build/pb", "/get_user_info.rs"));
//   }
//   pub mod qot_common {
//     include!(concat!("../target/debug/build/pb", "/qot_common.rs"));
//   }
//   pub mod global_state {
//     include!(concat!("../target/debug/build/pb", "/get_global_state.rs"));
//   }
//   pub mod qot_get_market_state {
//     include!(concat!("../target/debug/build/pb", "/qot_get_market_state.rs"));
//   }
//   pub mod qot_get_security_snapshot {
//     include!(concat!("../target/debug/build/pb", "/qot_get_security_snapshot.rs"));
//   }
//   pub mod qot_get_user_security {
//     include!(concat!("../target/debug/build/pb", "/qot_get_user_security.rs"));
//   }
//   pub mod qot_get_user_security_group {
//     include!(concat!("../target/debug/build/pb", "/qot_get_user_security_group.rs"));
//   }
//   pub mod qot_modify_user_security {
//     include!(concat!("../target/debug/build/pb", "/qot_modify_user_security.rs"));
//   }
// }

pub mod pb;
pub use pb::protocol::ProtoId;
