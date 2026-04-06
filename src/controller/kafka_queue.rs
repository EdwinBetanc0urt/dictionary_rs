use std::env;
use std::sync::Arc;
use std::time::Duration;

use rdkafka::{Message, TopicPartitionList, consumer::{CommitMode, Consumer, stream_consumer::StreamConsumer}};
use rdkafka::message::OwnedMessage;
use tokio::sync::{mpsc, Semaphore};

use crate::controller::{kafka_consumer::CustomContext, opensearch_document::DocumentProvider};
use crate::{controller::{kafka_consumer::create_consumer, opensearch_index::process_index}, models::{browser::BrowserDocument, form::FormDocument, menu_item::MenuItemDocument, menu_tree::MenuTreeDocument, process::ProcessDocument, role::RoleDocument, window::WindowDocument}};

struct MessageProcessor {
	commit_tx: mpsc::Sender<OwnedMessage>,
	semaphore: Arc<Semaphore>,
}

impl MessageProcessor {
	// Creates a new MessageProcessor and returns its commit receiver.
	fn new(concurrency_limit: usize) -> (Self, mpsc::Receiver<OwnedMessage>) {
		let (commit_tx, commit_rx) = mpsc::channel(1000);
		(Self {
			commit_tx,
			semaphore: Arc::new(Semaphore::new(concurrency_limit)),
		}, commit_rx)
	}
}

pub async fn consume_queue() {
	let kafka_host = env::var("KAFKA_HOST").unwrap_or_else(|_| "127.0.0.1:9092".to_string());
	let kafka_group = env::var("KAFKA_GROUP").unwrap_or_else(|_| "default".to_string());
	let kafka_queues = env::var("KAFKA_QUEUES").unwrap_or_else(|_| {
		"browser form process window menu_item menu_tree role".to_string()
	});
	let commit_batch_size = env::var("KAFKA_COMMIT_BATCH_SIZE")
		.map(|s| s.parse().unwrap_or(100))
		.unwrap_or(100);

	let commit_interval_secs = env::var("KAFKA_COMMIT_INTERVAL_SECS")
		.map(|s| s.parse().unwrap_or(5))
		.unwrap_or(5);

	let concurrency_limit = env::var("KAFKA_CONCURRENCY_LIMIT")
		.map(|s| s.parse().unwrap_or(10))
		.unwrap_or(10);

	let topics_list: Vec<&str> = kafka_queues.split_whitespace().collect();
	log::info!("Kafka queue to Subscribe: {:?}", kafka_host);
	log::info!("Kafka Topics to Subscribe: {:?}", topics_list);

	match create_consumer(&kafka_host, &kafka_group, &topics_list) {
		Ok(consumer) => {
			let consumer_arc = Arc::new(consumer);
			let (processor_struct, commit_rx) = MessageProcessor::new(concurrency_limit);
			let processor = Arc::new(processor_struct);

			// Spawn the worker for asynchronous batch commits
			tokio::spawn(commit_worker(
				Arc::clone(&consumer_arc),
				commit_rx,
				commit_batch_size,
				commit_interval_secs,
			));

			loop {
				match consumer_arc.recv().await {
					Err(e) => log::error!("Kafka error: {}", e),
					Ok(message) => {
						let owned_message = message.detach();
						let proc_clone = Arc::clone(&processor);

						// Process each message in a separate task
						tokio::spawn(async move {
							let _permit = proc_clone.semaphore.acquire().await.unwrap();
							if process_single_message(&owned_message).await {
								if let Err(e) = proc_clone.commit_tx.send(owned_message).await {
									log::error!("Failed to send message to commit worker: {}", e);
								}
							}
						});
					}
				}
			}
		}
		Err(error) => log::error!("Consume Queue Error {}", error),
	}
}

// Worker function to handle batch commits to Kafka
async fn commit_worker(
	consumer: Arc<StreamConsumer<CustomContext>>,
	mut commit_rx: mpsc::Receiver<OwnedMessage>,
	batch_size: usize,
	interval_secs: u64,
) {
	let mut last_commit_time = std::time::Instant::now();
	let mut count = 0;

	while let Some(message) = commit_rx.recv().await {
		count += 1;
		// Perform commit if batch size is reached or interval has elapsed
		if count >= batch_size || last_commit_time.elapsed() > Duration::from_secs(interval_secs) {
			let mut tpl = TopicPartitionList::new();
			if let Err(e) = tpl.add_partition_offset(
				message.topic(),
				message.partition(),
				rdkafka::Offset::Offset(message.offset() + 1),
			) {
				log::error!("Error creating commit list: {}", e);
			} else if let Err(e) = consumer.commit(&tpl, CommitMode::Async) {
				log::error!("Commit failed: {}", e);
			} else {
				log::debug!("Successfully committed batch of {} messages", count);
			}
			count = 0;
			last_commit_time = std::time::Instant::now();
		}
	}
}

// Processes a single message by determining its topic and calling the appropriate parser
async fn process_single_message(message: &OwnedMessage) -> bool {
	let topic = message.topic();
	let payload = match message.payload_view::<str>() {
		Some(Ok(s)) => s,
		_ => return false,
	};

	let key = message.key_view::<str>().and_then(|k| k.ok()).unwrap_or("");
	let event_type = key.replace("\"", "");

	// Determine the document type based on the topic and process accordingly
	match topic {
		"menu_item" => parse_and_process::<MenuItemDocument>(payload, event_type).await,
		"menu_tree" => parse_and_process::<MenuTreeDocument>(payload, event_type).await,
		"role" => parse_and_process::<RoleDocument>(payload, event_type).await,
		"process" => parse_and_process::<ProcessDocument>(payload, event_type).await,
		"browser" => parse_and_process::<BrowserDocument>(payload, event_type).await,
		"window" => parse_and_process::<WindowDocument>(payload, event_type).await,
		"form" => parse_and_process::<FormDocument>(payload, event_type).await,
		_ => {
			log::warn!("Topic {:?} not recognized", topic);
			false
		}
	}
}

// Generic function to deserialize and index a document
async fn parse_and_process<T>(payload: &str, event_type: String) -> bool
where
	T: serde::de::DeserializeOwned + DocumentProvider,
{
	match serde_json::from_str::<T>(payload) {
		Ok(doc) => {
			if let Some(inner) = doc.get_document() {
				match process_index(event_type, inner).await {
					Ok(_) => true,
					Err(e) => {
						log::error!("Error indexing document: {}", e);
						false
					}
				}
			} else {
				false
			}
		}
		Err(e) => {
			log::error!("Error deserializing payload: {}", e);
			false
		}
	}
}
