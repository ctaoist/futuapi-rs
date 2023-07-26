use futuapi::FutuApi;
use log::LevelFilter;
use simple_logger::SimpleLogger;

pub async fn init() -> FutuApi {
  SimpleLogger::new().with_level(LevelFilter::Trace).init().unwrap();
  let mut api = FutuApi::new();
  api.connect("127.0.0.1:1234").await.unwrap();
  api
}
