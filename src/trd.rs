/// 交易模块
///
/// 通过外网访问 FutuOpenD 需要加密
use super::{common, pb, FutuApi, ProtoId};
use prost::Message;

impl FutuApi {
  /// 获取交易业务账户列表
  ///
  pub async fn get_account_list(&self, category: common::TrdCategory, need_general: bool) -> Result<Vec<common::TrdAccount>, Box<dyn std::error::Error>> {
    let req = pb::trd_get_acc_list::Request {
      c2s: pb::trd_get_acc_list::C2s {
        user_id: 0,
        trd_category: Some(category as i32),
        need_general_sec_account: Some(need_general),
      },
    };

    let m = self.send(req, ProtoId::TrdGetAccList).await.unwrap();

    let m = pb::trd_get_acc_list::Response::decode(&m[..])?;

    match pb::common::RetType::from_i32(m.ret_type).unwrap() {
      pb::common::RetType::Succeed => {
        let tas = m.s2c.unwrap().acc_list;
        log::debug!("{:?}", tas);
        Ok(tas)
      }
      _ => {
        log::error!("get_user_security_group Error: {}", m.ret_msg());
        Err("get_user_security_group Error")?
      }
    }

    // let tas = m.s2c.unwrap().acc_list;
    // log::debug!("{:?}", tas);

    // Ok(tas)
  }
}
