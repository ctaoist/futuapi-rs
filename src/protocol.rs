use std::any::Any;
use std::collections::HashMap;
use std::io::Error;
use std::sync::{Mutex, RwLock};
// use futures::lock::{Mutex, RwLock};

// use bytes::Buf as BytesBuf;
use bytes::Bytes;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
// use parking_lot::{Mutex, RwLock};
use prost::bytes::{Buf, BufMut};
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use super::{
  pb::{init_connect, keep_alive},
  ProtoId,
};

/// 2+4+1+1+4+4+20+8
pub const HEADER_LENGTH: usize = 44;

#[derive(Debug, Default)]
pub struct APIProtoHeader {
  /// 包头起始标志，固定为“FT”
  pub header_flag: [u8; 2],
  /// 协议ID
  pub proto_id: u32,
  /// 协议格式类型，0为Protobuf格式，1为Json格式
  pub proto_fmt_type: u8,
  /// 协议版本，用于迭代兼容, 目前填0
  pub proto_ver: u8,
  /// 包序列号，用于对应请求包和回包, 要求递增
  pub serial_no: u32,
  /// 包体长度
  pub body_len: u32,
  /// 包体原始数据(解密后)的SHA1哈希值
  pub body_sha1: [u8; 20],
  /// 保留8字节扩展
  pub reserved: [u8; 8],
}

impl APIProtoHeader {
  pub fn new() -> Self {
    APIProtoHeader {
      header_flag: [b'F', b'T'],
      proto_fmt_type: 0,
      proto_ver: 0,
      ..Default::default()
    }
  }

  fn check_header_flag(&self) -> bool {
    self.header_flag == [b'F', b'T']
  }

  pub fn from_bytes(b: &mut [u8]) -> Self {
    let mut h = APIProtoHeader::new();
    let mut b = &b[..];
    b.copy_to_slice(&mut h.header_flag);
    h.proto_id = b.get_u32_le();
    h.proto_fmt_type = b.get_u8();
    h.proto_ver = b.get_u8();
    h.serial_no = b.get_u32_le();
    h.body_len = b.get_u32_le();
    b.copy_to_slice(&mut h.body_sha1);
    // b.copy_to_slice(&mut h.header_flag);
    h
  }

  fn encoded_bytes(&self) -> usize {
    // 2+4+1+1+4+4+20+8
    HEADER_LENGTH
  }

  fn as_bytes(&self) -> [u8; HEADER_LENGTH] {
    let mut v = Vec::with_capacity(self.encoded_bytes());
    v.put_slice(&self.header_flag);
    v.put_u32_le(self.proto_id);
    v.push(self.proto_fmt_type);
    v.push(self.proto_ver);
    v.put_u32_le(self.serial_no);
    v.put_u32_le(self.body_len);
    v.put_slice(&self.body_sha1);
    v.put_slice(&self.reserved);
    v.try_into().unwrap()
  }
}

#[derive(Debug)]
pub struct FutuEncoder<M: prost::Message> {
  pub proto: ProtoId,
  pub serial: u32,
  pub msg: M,
}

impl<M: prost::Message> FutuEncoder<M> {
  pub fn new(proto: ProtoId, serial: u32, msg: M) -> Self {
    Self { proto, serial, msg }
  }

  pub fn encode_to_vec(&self) -> Bytes {
    let m = self.msg.encode_to_vec();
    let mut hasher = sha1(m.as_ref());

    let mut h = APIProtoHeader {
      header_flag: [b'F', b'T'],
      proto_id: self.proto as u32,
      proto_fmt_type: 0,
      proto_ver: 0,
      serial_no: self.serial,
      body_len: self.msg.encoded_len() as u32,
      ..Default::default()
    };
    hasher.result(&mut h.body_sha1);

    let mut v = bytes::BytesMut::with_capacity(HEADER_LENGTH + m.len());
    // let mut v = Bytes::with_capacity(HEADER_LENGTH + m.len());
    v.put_slice(h.as_bytes().as_ref());
    v.put_slice(m.as_slice());
    v.into()
  }
}

fn sha1(msg: &[u8]) -> Sha1 {
  let mut hasher = Sha1::new();
  hasher.input(msg);
  hasher
}

// pub struct FutuDecoder {}

// impl FutuDecoder {
//   pub fn new() -> Self {
//     Self {}
//   }

//   pub fn decode(&self) {
//     let h = APIProtoHeader::new();
//   }
// }

#[derive(Debug)]
pub struct Conn {
  /// TCP 读
  pub connr: tokio::sync::Mutex<Option<OwnedReadHalf>>,
  /// TCP 写
  pub connw: tokio::sync::Mutex<Option<OwnedWriteHalf>>,
  /// 地址
  addr: tokio::sync::RwLock<String>,
  /// online 是否掉线
  online: RwLock<bool>,
  /// 包序列号
  serial: Mutex<u32>,
  /// 重连成功 channel
  reconn_rx: tokio::sync::RwLock<mpsc::Receiver<bool>>,
  reconn_tx: mpsc::Sender<bool>,
  //
  // pub m: RwLock<HashMap<u32, Box<mpsc::Sender<dyn Message>>>>,
  m: RwLock<HashMap<u32, mpsc::Sender<Bytes>>>,
}

// unsafe impl Send for Conn {}
// unsafe impl Sync for Conn {}
// unsafe impl Send for mpsc::Sender<Bytes> {}

impl Default for Conn {
  fn default() -> Self {
    let (tx, rx) = mpsc::channel::<bool>(1);
    Self {
      reconn_rx: tokio::sync::RwLock::new(rx),
      reconn_tx: tx,
      connr: Default::default(),
      connw: Default::default(),
      addr: Default::default(),
      online: RwLock::new(false),
      serial: Mutex::new(0),
      m: Default::default(),
    }
  }
}

impl Conn {
  pub async fn set_addr(&self, addr: &str) {
    let mut a = self.addr.write().await;
    *a = addr.to_string();
  }

  pub fn get_serial(&self) -> u32 {
    let mut s = self.serial.lock().unwrap();
    *s += 1;
    *s
  }

  /// 修改在线状态
  fn set_online(&self, status: bool) {
    let mut o = self.online.write().unwrap();
    *o = status;
  }

  pub fn online(&self) -> bool {
    let o = self.online.read().unwrap();
    *o
  }

  pub async fn dail(&self) -> Result<(), Error> {
    let addr = self.addr.read().await;
    // let addr = self.addr.as_str();
    // let mut c = self.conn.borrow_mut();
    // let mut c = self.conn.lock().await;
    // *c = Some(TcpStream::connect(addr).await?);
    let c = TcpStream::connect(addr.as_str()).await?.into_split();
    self.set_online(true);

    let mut w = self.connw.lock().await;
    let mut r = self.connr.lock().await;
    *r = Some(c.0);
    *w = Some(c.1);
    Ok(())
  }

  pub async fn add_chain(&self, serial: u32, tx: mpsc::Sender<Bytes>) {
    let mut m = self.m.write().unwrap();
    (*m).insert(serial, tx);
  }

  pub fn get_chain(&self, serial: u32) -> mpsc::Sender<Bytes> {
    let m = self.m.read().unwrap();
    m.get(&serial).unwrap().clone()
  }

  pub async fn del_chain(&self, serial: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut m = self.m.write().unwrap();
    (*m).remove(&serial);
    Ok(())
  }

  /// 返回包头 [APIProtoHeader]
  #[deprecated(note = "不应该单独使用")]
  async fn read_header(&self) -> Result<APIProtoHeader, Box<dyn std::error::Error>> {
    // let mut c = self.conn.borrow_mut();
    let mut c = self.connr.lock().await;
    let conn = c.as_mut().unwrap();

    // 读取 header
    conn.readable().await?;
    let mut buf = [0; 44];
    conn.read_exact(&mut buf).await?;
    let h = APIProtoHeader::from_bytes(&mut buf);
    log::trace!("Response APIProtoHeader: {:?}", h);
    Ok(h)
  }

  /// 返回包体和包头 [(Bytes, APIProtoHeader)]
  pub async fn read_message_with_header(&self) -> Result<(Bytes, APIProtoHeader), Error> {
    // let mut c = self.conn.borrow_mut();
    let mut c = self.connr.lock().await;
    let conn = c.as_mut().unwrap();

    // 读取 header
    conn.readable().await?;
    let mut buf = [0; 44];
    match conn.read_exact(&mut buf).await {
      Ok(_) => (),
      Err(e) => {
        log::error!("{}", e);
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
          log::info!("掉线了，等待重连中.......");
          self.set_online(false);
        }
        return Err(e.into());
      }
    };

    let h = APIProtoHeader::from_bytes(&mut buf);
    log::trace!("Response APIProtoHeader: {:?}", h);

    // 读取 Resp，返回 Vec<8u>
    let mut buf = bytes::BytesMut::with_capacity(h.body_len as usize);
    buf.put_bytes(0, h.body_len as usize);
    // let mut buf = vec![0; h.body_len as usize];
    // conn.readable().await.unwrap();
    conn.read_exact(&mut buf).await?;

    if !h.check_header_flag() {
      log::error!("API Proto Header Error: {:?}", h.header_flag);
      return Err(Error::new(std::io::ErrorKind::Other, "API Proto Header Error"));
    }

    Ok((buf.into(), h))
  }

  /// 返回包体 [Vec<u8>]
  pub async fn read_message(&self) -> Result<Bytes, Box<dyn std::error::Error>> {
    Ok(self.read_message_with_header().await?.0)
  }

  pub async fn write_message(&self, req: impl Message, proco_id: ProtoId, serial: u32, tx: Option<mpsc::Sender<Bytes>>) -> Result<(), Box<dyn std::error::Error>> {
    if !self.online() {
      return Ok(());
    }

    // let mut c = self.conn.borrow_mut();
    let mut c = self.connw.lock().await;
    // *c = Some(TcpStream::connect("127.0.0.1:1234").await.unwrap());
    let conn = c.as_mut().unwrap();
    let ready = conn.ready(Interest::WRITABLE).await?;

    if let Some(tx) = tx {
      self.add_chain(serial, tx).await;
    }

    if ready.is_writable() {
      let encoder = FutuEncoder::new(proco_id, serial, req);
      log::trace!("Write Message: {:?}", encoder);

      match conn.write_all(encoder.encode_to_vec().as_ref()).await {
        Ok(_) => (),
        Err(e) => {
          log::error!("{}", e);
          self.del_chain(serial).await?;
          if e.kind() == std::io::ErrorKind::BrokenPipe {
            log::info!("掉线了，等待重连中.......");
            self.set_online(false);
          }
          return Err(e.into());
        }
      };
    }
    Ok(())
  }

  pub async fn recv_loop(&self) {
    loop {
      if !self.online() {
        let mut rx = self.reconn_rx.write().await;
        rx.recv().await;
        // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        continue;
      }
      if let Ok((v, h)) = self.read_message_with_header().await {
        match ProtoId::from_i32(h.proto_id as i32).unwrap_or_default() {
          ProtoId::None => continue,
          ProtoId::KeepAlive => self.handle_heart_beat(v),
          _ => {
            let tx = self.get_chain(h.serial_no);
            tx.send(v).await.unwrap_or_else(|e| {
              log::error!("{}", e);
            });
            // self.del_chain(h.serial_no).await.unwrap();
          }
        }
      }
    }
  }

  /// 自动重连
  pub async fn auto_reconnect(&self) {
    log::debug!("已开启自动重连");
    let mut i = 1u64;
    loop {
      if !self.online() {
        log::info!("掉线重连中......");
        match self.dail().await {
          Ok(_) => {
            self.reconn_tx.send(true).await.unwrap();
            log::info!("重连成功!");
            i = 1;
          }
          Err(e) => {
            log::error!("重连失败: {}, 等待下一次重连.....", e);
            if i < 3 {
              i += 1;
            }
          }
        };
      }
      tokio::time::sleep(std::time::Duration::from_secs(i * 10)).await;
    }
  }

  pub async fn heart_beat(&self, keep_alive_interval: i32) {
    loop {
      if self.online() {
        let req = keep_alive::Request {
          c2s: keep_alive::C2s {
            time: time::OffsetDateTime::now_utc().unix_timestamp(),
          },
        };
        let serial = self.get_serial(); //self.get_serial();
        log::trace!("send heart beat package");
        self.write_message(req, ProtoId::KeepAlive, serial, None).await.unwrap();
      }
      tokio::time::sleep(std::time::Duration::from_secs(keep_alive_interval as u64)).await;
    }
  }

  fn handle_heart_beat(&self, v: Bytes) {
    let m = keep_alive::Response::decode(&v[..]).unwrap();
    let m = m.s2c.unwrap();
    log::trace!("收到心跳包, timestamp: {}", m.time);
  }

  #[deprecated(note = "Please use the [read_message()] function instead")]
  async fn read_message_any(&self) -> impl Any {
    // let mut c = self.conn.borrow_mut();
    let mut c = self.connr.lock().await;
    let conn = c.as_mut().unwrap();

    // 读取 header
    conn.readable().await.unwrap();
    let mut buf = [0; 44];
    conn.read_exact(&mut buf).await.unwrap();
    let h = APIProtoHeader::from_bytes(&mut buf);
    log::trace!("{:?}", h);
    // conn.readable().await.unwrap();

    // 读取 Resp
    let mut buf = vec![0; h.body_len as usize];
    conn.read_exact(&mut buf).await.unwrap();
    let resp = init_connect::Response::decode(&buf[..]).unwrap();
    log::debug!("Response: {:?}", resp);
    // *msg = resp.s2c.unwrap();
    resp.s2c.unwrap()

    // resp.s2c.unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_sha1() {
    let s = String::from("abcd");
    let mut h = sha1(s.as_bytes());
    let mut b = [0; 20];
    h.result(&mut b);
    println!("test_sha1: {}, {}, {}", h.output_bits(), h.output_bytes(), h.result_str());
    assert_eq!(h.result_str(), "81fe8bfe87576c3ecb22426f8e57847382917acf");
  }
}
