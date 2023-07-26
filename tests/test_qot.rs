mod connect;

#[tokio::test]
async fn test_qot() {
  let api = connect::init().await;

  api.get_security_market_state("sh000001,HZ399001").await.unwrap();
  api.get_security_snapshot("sh000001,HZ399001").await.unwrap();

  api.get_user_security_group(futuapi::SecurityGroupType::All).await.unwrap();
  api.get_user_security("全部").await.unwrap();

  tokio::time::sleep(tokio::time::Duration::from_secs(301)).await;
}
