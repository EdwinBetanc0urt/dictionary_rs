use std::env;
use dictionary_rs::controller::{api_rest::routes, kafka_queue::consume_queue};
use dotenv::dotenv;
use salvo::{Listener, Server, conn::{TcpListener, tcp::TcpAcceptor}};
extern crate serde_json;
use simple_logger::SimpleLogger;
use futures::future::join_all;


#[tokio::main]
async fn main() {
	dotenv().ok();
	SimpleLogger::new().env().init().unwrap();

	let port: String = match env::var("PORT") {
		Ok(value) => value,
		Err(_) => {
			log::warn!("Variable `PORT` Not found from enviroment, as default 7878");
			"7878".to_owned()
		}.to_owned()
	};

	let host: String = "0.0.0.0:".to_owned() + &port;
	log::info!("Server Address: {:?}", host.clone());
	let acceptor: TcpAcceptor = TcpListener::new(host).bind().await;

	let mut futures: Vec<tokio::task::JoinHandle<()>> = Vec::new();
	futures.push(
		tokio::spawn(
			async move { Server::new(acceptor).serve(routes()).await; }
		)
	);

	// Kafka Queue
	let kafka_enabled: String = match env::var("KAFKA_ENABLED") {
		Ok(value) => value,
		Err(_) => {
			log::warn!("Variable `KAFKA_ENABLED` Not found from enviroment, as default Y");
			"Y".to_owned()
		}.to_owned()
	};
	if kafka_enabled.trim().eq("Y") {
		log::info!("Kafka Consumer is enabled");
		futures.push(
			tokio::spawn(
				async move { consume_queue().await; }
			)
		);
	} else {
		log::info!("Kafka Consumer is disabled");
	}

	join_all(futures).await;
}
