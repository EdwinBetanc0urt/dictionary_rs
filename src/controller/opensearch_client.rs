use std::env;

use opensearch::OpenSearch;
use opensearch::http::Url;
use opensearch::http::transport::{SingleNodeConnectionPool, Transport, TransportBuilder};


pub fn create_opensearch_client() -> Result<OpenSearch, String> {
	let opensearch_url: String =  match env::var("OPENSEARCH_URL") {
		Ok(value) => value.clone(),
		Err(_) => {
			log::warn!("Variable `OPENSEARCH_URL` Not found from enviroment, loaded with `default` value");
			"http://localhost:9200".to_owned()
		}.to_owned(),
	};
	let url: Url = match Url::parse(&opensearch_url) {
		Ok(value) => value,
		Err(error) => {
			return Err(error.to_string());
		},
	};
	let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(url);
	let transport: Transport = match TransportBuilder::new(conn_pool)
		.disable_proxy()
		// .auth(Credentials::Basic("admin".to_owned(), "admin".to_owned()))
		.build() {
			Ok(value) => value,
			Err(error) => {
				return Err(error.to_string());
			},
		};
	Ok(OpenSearch::new(transport))
}
