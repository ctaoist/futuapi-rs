mod connect;

#[tokio::test]
async fn test_connect() {
  let api = connect::init().await;
  api.get_user_info().await.unwrap();
  api.get_global_state().await.unwrap();

  tokio::time::sleep(tokio::time::Duration::from_secs(301)).await;
  // std::thread::sleep(std::time::Duration::from_secs(30));
}
