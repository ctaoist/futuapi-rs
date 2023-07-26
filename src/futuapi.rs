use bytes::Bytes;
use prost::Message;
use std::sync::Arc;
use tokio::sync::mpsc;

// #[macro_use]
// use downcast_rs::{Downcast, impl_downcast};
// impl_downcast!(Message);

use super::{
  pb::{self, common, get_user_info::UserInfoField, init_connect},
  protocol::Conn,
  ProtoId,
};
use from_into_derive::FromIntoStruct;

#[derive(Debug, FromIntoStruct)]
pub struct UserInfo(pub pb::get_user_info::S2c);

#[derive(Debug, FromIntoStruct)]
pub struct GlobalState(pub pb::get_global_state::S2c);

// pub type FutuApi = init_connect::S2c;
#[derive(Debug, Default)]
pub struct FutuApi {
  /// 客户端 id, 默认 "RustFutuApi"
  pub client_id: String,
  /// TCP连接
  pub conn: Arc<Conn>,
  /// TCP Timeout, 默认 5 秒
  pub timeout: u64,
  /// 掉线是否重连，默认重连
  pub auto_reconnect: bool,
  // 包序列号
  // pub serial: RwLock<u32>,
  /// FutuOpenD的版本号
  pub server_ver: i32,
  /// FutuOpenD登陆的牛牛用户ID
  pub login_user_id: u64,
  /// 此连接的连接ID，连接的唯一标识
  pub conn_id: u64,
  /// 此连接后续AES加密通信的Key，固定为16字节长字符串
  pub conn_aes_key: ::prost::alloc::string::String,
  /// 心跳保活间隔
  pub keep_alive_interval: i32,
  /// AES加密通信CBC加密模式的iv，固定为16字节长字符串
  pub aes_cb_civ: ::core::option::Option<::prost::alloc::string::String>,
  /// 用户类型，牛牛用户或MooMoo用户
  pub user_attribution: common::UserAttribution,
}

impl FutuApi {
  pub fn new() -> Self {
    return FutuApi {
      client_id: String::from("RustFutuAPI"),
      timeout: 5,
      auto_reconnect: true,
      ..Default::default()
    };
  }

  pub async fn connect(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    // let mut c = self.conn.write().unwrap();
    self.conn.set_addr(addr).await;
    self.conn.dail().await?;

    let c = self.conn.clone();
    tokio::spawn(async move {
      c.recv_loop().await;
    });

    let req = init_connect::Request {
      c2s: init_connect::C2s {
        client_ver: 1,
        client_id: self.client_id.clone(),
        recv_notify: Some(true),
        packet_enc_algo: Some(common::PacketEncAlgo::None as i32),
        push_proto_fmt: Some(common::ProtoFmt::Protobuf as i32),
        programming_language: Some(String::from("rust")),
      },
    };

    let (tx, mut rx) = mpsc::channel::<Bytes>(1);
    let serial = self.conn.get_serial();
    self.conn.write_message(req, ProtoId::InitConnect, serial, Some(tx)).await?;

    let m = tokio::time::timeout(std::time::Duration::from_secs(self.timeout), rx.recv()).await?.unwrap();
    self.conn.del_chain(serial).await?;

    // let m = self.conn.read_message().await?;
    let m = init_connect::Response::decode(&m[..])?;
    let m = m.s2c.unwrap();
    log::debug!("Response InitConnect msg: {:?}", m);

    // 填充到 [FutuApi] 中
    self.user_attribution = common::UserAttribution::from_i32(m.user_attribution()).unwrap();
    self.conn_id = m.conn_id;
    self.conn_aes_key = m.conn_aes_key;
    self.server_ver = m.server_ver;
    self.login_user_id = m.login_user_id;
    self.keep_alive_interval = m.keep_alive_interval;
    self.aes_cb_civ = m.aes_cb_civ;
    log::debug!("{:?}", self);

    let c = self.conn.clone();
    tokio::spawn(async move {
      c.heart_beat(m.keep_alive_interval).await;
    });

    if self.auto_reconnect {
      let c = self.conn.clone();
      tokio::spawn(async move {
        c.auto_reconnect().await;
      });
    }

    log::info!("FutuOpenD 连接成功!");

    Ok(())
    // if let Some(m) = m.as_any().downcast_ref::<init_connect::S2c>() {
    // }
  }

  pub async fn send(&self, req: impl Message, proto_id: ProtoId) -> Result<Bytes, Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel::<Bytes>(1);
    let serial = self.conn.get_serial();

    self.conn.write_message(req, proto_id, serial, Some(tx)).await?;

    let m = tokio::time::timeout(std::time::Duration::from_secs(self.timeout), rx.recv()).await?.unwrap();
    self.conn.del_chain(serial).await?;

    Ok(m)
  }

  pub async fn get_user_info(&self) -> Result<UserInfo, Box<dyn std::error::Error>> {
    let req = pb::get_user_info::Request {
      c2s: pb::get_user_info::C2s {
        flag: Some(UserInfoField::Basic as i32 + UserInfoField::Api as i32 | UserInfoField::QotRight as i32),
      },
    };

    let m = self.send(req, ProtoId::GetUserInfo).await?;

    let m = pb::get_user_info::Response::decode(&m[..])?;
    let u = UserInfo::from_struct(m.s2c.unwrap());
    log::debug!("{:?}", u);

    Ok(u)
  }

  pub async fn get_global_state(&self) -> Result<GlobalState, Box<dyn std::error::Error>> {
    let req = pb::get_global_state::Request {
      c2s: pb::get_global_state::C2s { user_id: 0 },
    };

    let m = self.send(req, ProtoId::GetGlobalState).await?;

    let m = pb::get_global_state::Response::decode(&m[..])?;
    let gs = GlobalState::from_struct(m.s2c.unwrap());
    log::debug!("{:?}", gs);

    Ok(gs)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use log::LevelFilter;
  use simple_logger::SimpleLogger;

  #[tokio::test]
  async fn test_connect() {
    SimpleLogger::new().with_level(LevelFilter::Trace).init().unwrap();

    let mut api = FutuApi::new();
    api.connect("127.0.0.1:1234").await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(31)).await;
  }
}
