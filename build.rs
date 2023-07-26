use std::io::Result;

fn main() -> Result<()> {
  // env::set_var("OUT_DIR", "target/pb");
  let out_dir = "src/pb";
  std::fs::create_dir_all(out_dir).unwrap();
  prost_build::Config::new().out_dir(out_dir).compile_protos(
    &[
      "Common.proto",
      "InitConnect.proto",
      "KeepAlive.proto",
      "GetUserInfo.proto",
      "Qot_Common.proto",
      "GetGlobalState.proto",
      "protocol.proto",
      "Qot_GetMarketState.proto",
      "Qot_GetSecuritySnapshot.proto",
      "Qot_GetUserSecurity.proto",
      "Qot_GetUserSecurityGroup.proto",
      "Qot_ModifyUserSecurity.proto",
      "Trd_GetAccList.proto",
    ],
    &["futu_proto/", "src/pb/"],
  )?;
  // prost_build::compile_protos(
  //   &[
  //     "futu_proto_7.1.3308/Common.proto",
  //     "futu_proto_7.1.3308/InitConnect.proto",
  //     "futu_proto_7.1.3308/KeepAlive.proto",
  //     "GetUserInfo.proto",
  //     "protocol.proto",
  //   ],
  //   &["futu_proto_7.1.3308/", "src/"],
  // )?;
  Ok(())
}
