use std::env;
use std::sync::OnceLock;
use std::time::Duration;

use opensearch::OpenSearch;
use opensearch::http::Url;
use opensearch::http::transport::{SingleNodeConnectionPool, Transport, TransportBuilder};

static OPENSEARCH_CLIENT: OnceLock<OpenSearch> = OnceLock::new();

pub fn create_opensearch_client() -> Result<OpenSearch, String> {
	// If the client is already initialized, return a clone (shares the same internal connection pool)
	if let Some(client) = OPENSEARCH_CLIENT.get() {
		return Ok(client.clone());
	}

	let opensearch_url: String = env::var("OPENSEARCH_URL").unwrap_or_else(|_| {
		log::warn!("Variable `OPENSEARCH_URL` not found, using default");
		"http://localhost:9200".to_string()
	});

	let timeout_secs = env::var("OPENSEARCH_CONNECTION_TIMEOUT_SECS")
		.map(|s| s.parse().unwrap_or(30))
		.unwrap_or(30);

	let url: Url = Url::parse(&opensearch_url).map_err(|e| e.to_string())?;
	let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(url);
	
	// Build the transport with a shared connection pool
	let transport: Transport = TransportBuilder::new(conn_pool)
		.disable_proxy()
		// .auth(Credentials::Basic("admin".to_owned(), "admin".to_owned()))
		.timeout(Duration::from_secs(timeout_secs))
		// By default, the underlying HTTP client handles connection pooling.
		// You can add .auth() or other configurations here if needed.
		.build()
		.map_err(|e| e.to_string())?;

	let client = OpenSearch::new(transport);

	// Attempt to set the singleton. If another thread won the race, use the existing instance.
	match OPENSEARCH_CLIENT.set(client.clone()) {
		Ok(_) => Ok(client),
		Err(_) => Ok(OPENSEARCH_CLIENT
			.get()
			.expect("Client should be initialized")
			.clone()),
	}
}
