use opensearch::{DeleteParts, GetParts, IndexParts, OpenSearch, SearchParts, http::response::Response};
use salvo::http::StatusCode;
use serde_json::Value;

use crate::controller::{opensearch_client::create_opensearch_client, opensearch_index::create_index_definition};


pub trait IndexDocument: Sync {
	//  A index definition for mapping
	fn mapping(self: &Self) -> serde_json::Value;
	//  Get data for insert
	fn data(self: &Self) -> serde_json::Value;
	//  Get index name for create and delete index definition
	fn index_name(self: &Self) -> String;
	//  Get Unique ID
	fn id(self: &Self) -> String;
	//  Make a search based on _search_value
	fn find(self: &Self, _search_value: String) -> serde_json::Value;
}

// Trait to extract the internal document from each model's wrappers
pub trait DocumentProvider {
	fn get_document(&self) -> Option<&dyn IndexDocument>;
}


pub async fn create(_document: &dyn IndexDocument) -> Result<bool, std::string::String> {
	let client: OpenSearch = create_opensearch_client()?;

	let _response: Result<bool, String> = create_index_definition(_document).await;
	let _response: bool = match _response {
		Ok(_) => true,
		Err(error) => {
			log::error!("{:?}", error);
			false
		}
	};
	match get_by_id(_document).await {
		Ok(_) => {
			match delete(_document).await {
				Ok(_) => {},
				Err(error) => log::error!("{:?}", error),
			};
		},
		Err(_) => {},
	};
	// Create
	let _response: Result<Response, opensearch::Error> = client
		.index(IndexParts::IndexId(&_document.index_name(), &_document.id()))
		.body(_document.data())
		.send()
		.await
	;
	let _response: Response = match _response {
		Ok(value) => value,
		Err(error) => {
			log::error!("{:?}", error);
			return Err(error.to_string());
		}
	};
	if !_response.status_code().is_success() {
		return Err(format!("Error inserting record {:?} {:?} {:?}", _document.index_name(), _document.id(), _response.text().await));
	}
	Ok(true)
}

pub async fn delete(_document: &dyn IndexDocument) -> Result<bool, std::string::String> {
	let client: OpenSearch = create_opensearch_client()?;

	// Delete
	let _response: Result<Response, opensearch::Error> = client
		.delete(DeleteParts::IndexId(&_document.index_name(), &_document.id()))
		.send()
		.await
	;

	match _response {
		Ok(value) => {
			let status: StatusCode = value.status_code();
			// For the ‘delete’ operation, the OpenSearch library often
			// considers 404 as ‘success’ if the document does not exist.
			if !status.is_success() && status.as_u16() != 404 {
				return Err(
					format!("Error deleting record {:?} {:?} {:?}", _document.index_name(), _document.id(), value.text().await)
				);
			}
			value
		},
		Err(error) => {
			log::error!("{:?}", error);
			return Err(error.to_string());
		}
	};
	Ok(true)
}

pub async fn find(_document: &dyn IndexDocument, _search_value: String, _from: i64, _size: i64) -> Result<Vec<Value>, std::string::String> {
	let client: OpenSearch = create_opensearch_client()?;

	// Get
	let _response: Result<opensearch::http::response::Response, opensearch::Error> = client
		.search(SearchParts::Index(&[&_document.index_name()]))
		.from(_from)
		.size(_size)
		.body(_document.find(_search_value))
		.send()
		.await
	;
	let response: Response = match _response {
		Ok(value) => value,
		Err(error) => {
			log::error!("{:?}", error);
			return Err(error.to_string());
		}
	};
	if !response.status_code().is_success() {
		return Err(format!("Error finding record {:?}", response.text().await));
	}
	let response_body: Value = match response.json::<Value>().await {
		Ok(response) => response,
		Err(error) => {
			log::error!("{:?}", error);
			return Err(error.to_string());
		},
	};

	let mut list: Vec::<Value> = Vec::new();
	for hit in response_body["hits"]["hits"].as_array().unwrap() {
		let value: Value = hit["_source"].to_owned();
		list.push(value)
	}
	Ok(list)
}

pub async fn find_from_dsl_body(_index_name: String, _body: serde_json::Value, _from: i64, _size: i64) -> Result<Vec<Value>, std::string::String> {
	let client: OpenSearch = create_opensearch_client()?;

	//  Get
	let _response: Result<opensearch::http::response::Response, opensearch::Error> = client
		.search(SearchParts::Index(&[&_index_name]))
		.from(_from)
		.size(_size)
		.body(_body)
		.send()
		.await
	;
	let response: Response = match _response {
		Ok(value) => value,
		Err(error) => {
			log::error!("{:?}", error);
			return Err(error.to_string());
		}
	};
	if !response.status_code().is_success() {
		return Err(format!("Error finding record {:?}", response.text().await));
	}
	let response_body: Value = match response.json::<Value>().await {
		Ok(response) => response,
		Err(error) => {
			log::error!("{:?}", error);
			return Err(error.to_string());
		},
	};

	let hits: &Vec<Value> = response_body["hits"]["hits"].as_array().unwrap();
	let mut list: Vec::<Value> = Vec::new();
	for hit in hits {
		let source: &Value = &hit["_source"];
		let value: Value = source.to_owned();
		list.push(value)
	}

	Ok(list)
}

pub async fn get_by_id(_document: &dyn IndexDocument) -> Result<Value, std::string::String> {
	let client: OpenSearch = create_opensearch_client()?;

	// Get
	let _response: Result<Response, opensearch::Error> = client
		.get(GetParts::IndexId(&_document.index_name(), &_document.id()))
		.send()
		.await
	;
	let _response: Response = match _response {
		Ok(value) => value,
		Err(error) => {
			log::error!("{:?}", error);
			return Err(error.to_string());
		}
	};
	if !_response.status_code().is_success() {
		return Err(format!("Error finding record by ID {:?}", _response.text().await));
	}
	let response_body: Value = match _response.json::<Value>().await {
		Ok(response) => {
			response["_source"].to_owned()
		},
		Err(error) => {
			log::error!("{:?}", error);
			return Err(error.to_string());
		},
	};
	Ok(response_body)
}
