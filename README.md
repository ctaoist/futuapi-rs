# 富途 OpenAPI Rust SDK

futu proto 当前版本：7.1.3308

## 简介

- Rust 语言封装的[富途牛牛OpenAPI](https://openapi.futunn.com/futu-api-doc/)。
- 掉线自动重连，使用了 `tokio::sync::{Mutex, RwLock}`，可能会对性能有点影响。

## USAGE

```rust
use futuapi::FutuApi;
use log::LevelFilter;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
  SimpleLogger::new().with_level(LevelFilter::Trace).init().unwrap();

  // 创建实例
  let mut api = FutuApi::new();
  // 连接到 FutuOpenD
  api.connect("127.0.0.1:1234").await.unwrap();
  // 获取全局状态
  api.get_global_state().await.unwrap();
  // 获取上证、深证指数市场状态
  api.get_security_market_state("sh000001,HZ399001").await.unwrap();
}
```

更多的例子可以看测试用例。

## API

协议框架已经搭好，完成了部分 API，参照现有的接口很容易补充新的接口，欢迎大佬 PR。

### 行情接口

- [x] 获取用户信息: `get_user_info`
- [x] 获取全局市场状态: `get_global_state`
- [ ] 订阅或反订阅
- [ ] 获取订阅信息
- [ ] 推送股票基本报价
- [ ] 推送 K 线
- [ ] 推送分时
- [ ] 推送逐笔
- [ ] 推送经纪队列
- [x] 获取股票市场状态: `get_security_market_state`
- [x] 获取股票快照: `get_security_snapshot`
- [ ] 获取股票基本报价
- [ ] 获取买卖盘
- [ ] 获取 K 线
- [ ] 获取分时
- [ ] 获取逐笔
- [ ] 获取经纪队列
- [ ] 获取资金流向
- [ ] 获取资金分布
- [ ] 获取股票所属板块
- [ ] 在线获取单只股票一段历史 K 线
- [ ] 在线获取单只股票复权信息
- [ ] 获取持股变化列表
- [x] 获取用户自选股列表: `get_user_security`
- [x] 获取用户自选股分组: `get_user_security_group`
- [ ] 调整用户自选股票
- [ ] 获取历史 K 线额度
- [ ] 设置到价提醒
- [ ] 获取到价提醒
- [ ] 到价提醒通知
- [ ] 获取市场交易日，在线拉取不在本地计算
- [ ] 获取条件选股
- [ ] 获取板块下的股票
- [ ] 获取板块集合下的板块
- [ ] 获取股票静态信息
- [ ] 获取新股
- [ ] 获取期权到期日
- [ ] 获取期权链
- [ ] 获取窝轮
- [ ] 获取正股相关股票
- [ ] 获取期货合约资料

### 交易接口

如果通过外网访问 FutuOpenD，使用交易接口需要配置加密，暂时还不支持加密。FutuOpenD 的监听地址为 `127.0.0.1` 时不需要加密。

- [x] 获取交易业务账户列表
- [ ] 解锁或锁定交易
- [ ] 获取账户资金
- [ ] 获取最大交易数量
- [ ] 获取账户持仓
- [ ] 获取融资融券数据
- [ ] 下单
- [ ] 修改订单
- [ ] 获取订单列表
- [ ] 获取历史订单列表
- [ ] 推送订单状态变动通知
- [ ] 订阅业务账户的交易推送数据
- [ ] 获取成交列表
- [ ] 获取历史成交列表
- [ ] 推送成交通知