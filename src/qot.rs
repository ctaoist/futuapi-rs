/// 行情模块
///
use prost::Message;

use super::{
  common::{SecruityGroupData, Security, SecurityMarketState, SecuritySnapshot, SecurityStaticInfo},
  pb::{self, qot_common, qot_get_market_state, qot_get_security_snapshot},
  FutuApi, ProtoId,
};

impl FutuApi {
  /// 获取股票市场状态
  ///
  /// `securites`: 股票列表, "SH000001,hz399001" 格式
  ///
  /// ## 接口限制
  ///
  /// - 每 30 秒内最多请求 10 次获取标的市场状态接口。
  /// - 每次请求的股票代码个数上限为 400 个。
  pub async fn get_security_market_state(&self, securites: &str) -> Result<Vec<SecurityMarketState>, Box<dyn std::error::Error>> {
    let l: Vec<qot_common::Security> = securites.split(",").map(|v| Security::from(v).into()).collect();
    let req = qot_get_market_state::Request {
      c2s: qot_get_market_state::C2s { security_list: l },
    };
    let m = self.send(req, ProtoId::QotGetMarketState).await?;

    let m = pb::qot_get_market_state::Response::decode(&m[..])?;
    let ss = m.s2c.unwrap().market_info_list;
    log::debug!("{:?}", ss);

    Ok(ss)
  }

  /// 获取股票、指数快照
  ///
  /// `securites`: 股票列表, "SH000001,hz399001" 格式
  ///
  /// ## 接口限制
  ///
  /// - 每 30 秒内最多请求 60 次快照。
  /// - 每次请求，接口参数 股票代码列表 支持传入的标的数量上限是 400 个。
  /// - 港股 BMP 权限下，单次请求的香港证券（含窝轮、牛熊、界内证）快照数量上限是 20 个。
  /// - 港股期权期货 BMP 权限下，单次请求的香港期货和期权的快照数量上限是 20 个。
  pub async fn get_security_snapshot(&self, securites: &str) -> Result<Vec<SecuritySnapshot>, Box<dyn std::error::Error>> {
    let l: Vec<qot_common::Security> = securites.split(",").map(|v| Security::from(v).into()).collect();
    let req = qot_get_security_snapshot::Request {
      c2s: qot_get_security_snapshot::C2s { security_list: l },
    };
    let m = self.send(req, ProtoId::QotGetSecuritySnapshot).await?;

    let m = pb::qot_get_security_snapshot::Response::decode(&m[..])?;
    let ss = m.s2c.unwrap().snapshot_list;
    log::debug!("{:?}", ss);

    Ok(ss)
  }

  /// 获取用户自选股分组
  ///
  /// `group_type`: [SecurityGroupType]
  ///
  /// ## 接口限制
  ///
  /// - 每 30 秒内最多请求 10 次获取自选股分组接口
  pub async fn get_user_security_group<T: Into<i32>>(&self, group_type: T) -> Result<Vec<SecruityGroupData>, Box<dyn std::error::Error>> {
    let req = pb::qot_get_user_security_group::Request {
      c2s: pb::qot_get_user_security_group::C2s { group_type: group_type.into() },
    };

    let m = self.send(req, ProtoId::QotGetUserSecurityGroup).await?;

    let m = pb::qot_get_user_security_group::Response::decode(&m[..])?;
    log::trace!("get_user_security_group Response: {:?}", m);

    match pb::common::RetType::from_i32(m.ret_type).unwrap() {
      pb::common::RetType::Succeed => {
        let us = m.s2c.unwrap().group_list;
        log::debug!("{:?}", us);
        Ok(us)
      }
      _ => {
        log::error!("get_user_security_group Error: {}", m.ret_msg());
        Err("get_user_security_group Error")?
      }
    }
  }

  /// 获取用户自选股列表
  ///
  /// `group_name`: 分组名
  ///
  /// 系统分组的中英文对应名称如下:
  ///
  /// | 中文 | 英文 |
  /// | -- | -- |
  /// | 全部 |	All
  /// | 沪深 | 	CN |
  /// | 港股 | 	HK |
  /// | 美股 | 	US |
  /// | 期权 | 	Options |
  /// | 港股期权 |	HK options |
  /// | 美股期权 |	US options |
  /// | 特别关注 |	Starred |
  /// | 期货 | 	Futures |
  ///
  /// ## Example
  ///
  /// ```
  /// use ...;
  ///
  /// let apt = /* */;
  /// apt.get_user_security("全部").await.unwrap();
  /// ```
  ///
  /// ## 接口限制
  ///
  /// - 每 30 秒内最多请求 10 次获取自选股列表接口
  /// - 不支持持仓（Positions），基金宝（Mutual Fund），外汇（Forex）分组的查询
  pub async fn get_user_security(&self, group_name: &str) -> Result<Vec<SecurityStaticInfo>, Box<dyn std::error::Error>> {
    let req = pb::qot_get_user_security::Request {
      c2s: pb::qot_get_user_security::C2s {
        group_name: group_name.to_owned(),
      },
    };

    let m = self.send(req, ProtoId::QotGetUserSecurity).await?;

    let m = pb::qot_get_user_security::Response::decode(&m[..])?;
    log::trace!("get_user_security Response: {:?}", m);

    if m.ret_type == pb::common::RetType::Succeed as i32 {
      let us = m.s2c.unwrap().static_info_list;
      log::debug!("{:?}", us);
      Ok(us)
    } else {
      log::error!("get_user_security Error: {}", m.ret_msg());
      Err("get_user_security Error")?
    }
  }
}
