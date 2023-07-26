mod connect;
use futuapi::common;

#[tokio::test]
async fn test_trd() {
  let api = connect::init().await;

  api.get_account_list(common::TrdCategory::Security, true).await.unwrap();
}
