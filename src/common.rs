use super::pb;

pub struct Security {
  pub market: pb::qot_common::QotMarket,
  pub code: String,
}

impl From<&str> for Security {
  fn from(s: &str) -> Self {
    let m = s[..2].to_uppercase();
    let market = match m.as_str() {
      "HK" => pb::qot_common::QotMarket::HkSecurity,
      "HZ" => pb::qot_common::QotMarket::CnszSecurity,
      "SH" => pb::qot_common::QotMarket::CnshSecurity,
      _ => pb::qot_common::QotMarket::Unknown,
    };
    Self { market, code: s[2..].to_string() }
  }
}

impl Into<pb::qot_common::Security> for Security {
  fn into(self) -> pb::qot_common::Security {
    pb::qot_common::Security {
      market: self.market as i32,
      code: self.code,
    }
  }
}

/// 股票、指数快照
pub type SecuritySnapshot = pb::qot_get_security_snapshot::Snapshot;
/// 股票市场状态，早盘、休市、午盘等
pub type SecurityMarketState = pb::qot_get_market_state::MarketInfo;
/// 股票静态信息
pub type SecurityStaticInfo = pb::qot_common::SecurityStaticInfo;
/// 股票静态基本信息
pub type SecurityStaticBasic = pb::qot_common::SecurityStaticBasic;
/// 自选股分组类型
pub type SecurityGroupType = pb::qot_get_user_security_group::GroupType;
/// 自选股分组数据
pub type SecruityGroupData = pb::qot_get_user_security_group::GroupData;

/// 交易品类
pub type TrdCategory = pb::trd_common::TrdCategory;
/// 交易业务账户结构
pub type TrdAccount = pb::trd_common::TrdAcc;
