use std::env;

use rdkafka::{Message, consumer::{CommitMode, Consumer}};

use crate::{controller::{kafka_consumer::create_consumer, opensearch_document::IndexDocument, opensearch_index::process_index}, models::{browser::BrowserDocument, form::FormDocument, menu_item::MenuItemDocument, menu_tree::MenuTreeDocument, process::ProcessDocument, role::RoleDocument, window::WindowDocument}};


pub async fn consume_queue() {
	let kafka_host: String = match env::var("KAFKA_HOST") {
		Ok(value) => value,
		Err(_) => {
			log::warn!("Variable `KAFKA_HOST` Not found from enviroment, loaded from local IP");
			"127.0.0.1:9092".to_owned()
		}.to_owned(),
	};
	log::info!("Kafka queue to Subscribe: {:?}", kafka_host.to_owned());

	let kafka_group: String = match env::var("KAFKA_GROUP") {
		Ok(value) => value,
		Err(_) => {
			log::warn!("Variable `KAFKA_GROUP` Not found from enviroment, loaded with `default` value");
			"default".to_owned()
		}.to_owned(),
	};
	let kafka_queues: String = match env::var("KAFKA_QUEUES") {
		Ok(value) => value.clone(),
		Err(_) => {
			log::warn!("Variable `KAFKA_QUEUES` Not found from enviroment, loaded with `default` value");
			"browser form process window menu_item menu_tree role".to_owned()
		}.to_owned()
	};

	let topics_list: Vec<&str> = kafka_queues.split_whitespace().collect();
	log::info!("Kafka Topics to Subscribe: {:?}", topics_list.to_owned());

	let consumer_result= create_consumer(&kafka_host, &kafka_group, &topics_list);
	match consumer_result {
		Ok(consumer) => {
			loop {
				match consumer.recv().await {
					Err(e) => log::error!("Kafka error: {}", e),
					Ok(message) => {
						let key: &str = match message.key_view::<str>() {
							None => "",
							Some(Ok(s)) => s,
							Some(Err(e)) => {
								log::error!("Error while deserializing message key: {:?}", e);
								""
							}
						};
						let event_type: String = key.replace("\"", "");
						let topic: &str = message.topic();
						if (topics_list.contains(&topic)) == false {
							log::warn!("Topic {:?} not allowed to be processed", topic);
							continue;
						}

						let payload: &str = match message.payload_view::<str>() {
							None => "",
							Some(Ok(s)) => s,
							Some(Err(e)) => {
								log::error!("Error while deserializing message payload: {:?}", e);
								""
							}
						};
						if topic == "menu_item" {
							let _document: MenuItemDocument = match serde_json::from_str(payload) {
								Ok(value) => value,
								Err(error) => {
									log::warn!("Topic: {:?}, {}", topic, error);
									MenuItemDocument {
										document: None
									}
								},
							};
							if _document.document.is_some() {
								let _menu_document: &dyn IndexDocument = &(_document.document.unwrap());
								match process_index(event_type, _menu_document).await {
									Ok(_) => consumer.commit_message(&message, CommitMode::Async).unwrap(),
									Err(error) => log::warn!("Document: {:?} {}", _menu_document.index_name(), error)
								}
							}
						} else if topic == "menu_tree" {
							let _document: MenuTreeDocument = match serde_json::from_str(payload) {
								Ok(value) => value,
								Err(error) => {
									log::warn!("Topic: {:?}, {}", topic, error);
									MenuTreeDocument {
										document: None
									}
								},
							};
							if _document.document.is_some() {
								let _menu_document: &dyn IndexDocument = &(_document.document.unwrap());
								match process_index(event_type, _menu_document).await {
									Ok(_) => consumer.commit_message(&message, CommitMode::Async).unwrap(),
									Err(error) => log::warn!("Document: {:?} {}", _menu_document.index_name(), error)
								}
							}
						} else if topic == "role" {
							let _document: RoleDocument = match serde_json::from_str(payload) {
								Ok(value) => value,
								Err(error) => {
									log::warn!("Topic: {:?}, {}", topic, error);
									RoleDocument {
										document: None
									}
								},
							};
							if _document.document.is_some() {
								let _menu_document: &dyn IndexDocument = &(_document.document.unwrap());
								match process_index(event_type, _menu_document).await {
									Ok(_) => consumer.commit_message(&message, CommitMode::Async).unwrap(),
									Err(error) => log::warn!("Document: {:?} {}", _menu_document.index_name(), error)
								}
							}
						} else if topic == "process" {
							let _document: ProcessDocument = match serde_json::from_str(payload) {
								Ok(value) => value,
								Err(error) => {
									log::warn!("Topic: {:?}, {}", topic, error);
									ProcessDocument {
										document: None
									}
								},
							};
							if _document.document.is_some() {
								let _process_document: &dyn IndexDocument = &(_document.document.unwrap());
								match process_index(event_type, _process_document).await {
									Ok(_) => consumer.commit_message(&message, CommitMode::Async).unwrap(),
									Err(error) => log::warn!("Document: {:?} {}", _process_document.index_name(), error)
								}
							}
						} else if topic == "browser" {
							let _document: BrowserDocument = match serde_json::from_str(payload) {
								Ok(value) => value,
								Err(error) => {
									log::warn!("Topic: {:?}, {}", topic, error);
									BrowserDocument {
										document: None
									}
								},
							};
							if _document.document.is_some() {
								let _browser_document: &dyn IndexDocument = &(_document.document.unwrap());
								match process_index(event_type, _browser_document).await {
									Ok(_) => consumer.commit_message(&message, CommitMode::Async).unwrap(),
									Err(error) => log::warn!("Document: {:?} {}", _browser_document.index_name(), error)
								}
							}
						} else if topic == "window" {
							let _document: WindowDocument = match serde_json::from_str(payload) {
								Ok(value) => value,
								Err(error) => {
									log::warn!("Topic: {:?}, {}", topic, error);
									WindowDocument {
										document: None
									}
								},
							};
							if _document.document.is_some() {
								let _window_document: &dyn IndexDocument = &(_document.document.unwrap());
								match process_index(event_type, _window_document).await {
									Ok(_) => consumer.commit_message(&message, CommitMode::Async).unwrap(),
									Err(error) => log::warn!("Document: {:?} {}", _window_document.index_name(), error)
								}
							}
						} else if topic == "form" {
							let _document: FormDocument = match serde_json::from_str(payload) {
								Ok(value) => value,
								Err(error) => {
									log::warn!("Topic: {:?}, {}", topic, error);
									FormDocument {
										document: None
									}
								},
							};
							if _document.document.is_some() {
								let _form_document: &dyn IndexDocument = &(_document.document.unwrap());
								match process_index(event_type, _form_document).await {
									Ok(_) => consumer.commit_message(&message, CommitMode::Async).unwrap(),
									Err(error) => log::warn!("Document: {:?} {}", _form_document.index_name(), error)
								}
							}
						}
						// TODO: Add token header
						// if let Some(headers) = message.headers() {
						// 	for header in headers.iter() {
						// 		log::info!("  Header {:#?}: {:?}", header.key, header.value);
						// 	}
						// }
					}
				};
			}
		},
		Err(error) => log::error!("Consume Queue Error {}", error),
	};
}
