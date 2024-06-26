use serde::{Deserialize, Serialize};
use salvo::prelude::*;
use serde_json::json;
use std::{io::ErrorKind, io::Error};

use crate::{controller::opensearch::{IndexDocument, get_by_id, find, exists_index}, models::client_index};

#[derive(Deserialize, Extractible, Debug, Clone)]
#[salvo(extract(default_source(from = "body")))]
pub struct FormDocument {
	pub document: Option<Form>
}

#[derive(Serialize, Debug, Clone)]
pub struct FormResponse {
	pub form: Option<Form>
}

#[derive(Serialize, Debug, Clone)]
pub struct FormsListResponse {
	pub forms: Option<Vec<Form>>
}

impl Default for FormResponse {
	fn default() -> Self {
		FormResponse {
			form: None
		}
	}
}

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct Form {
	pub uuid: Option<String>,
	pub id: Option<i32>,
	pub file_name: Option<String>,
	pub name: Option<String>,
	pub description: Option<String>,
	pub help: Option<String>,
	pub is_active: Option<bool>,
	//	Index
	pub index_value: Option<String>,
	pub language: Option<String>,
	pub client_id: Option<i32>,
	pub role_id: Option<i32>,
	pub user_id: Option<i32>
}

impl Default for Form {
	fn default() -> Self {
		Self {
			uuid: None,
			id: None,
			file_name: None,
			name: None,
			description: None,
			help: None,
			is_active: None,
			//	Index
			index_value: None,
			language: None,
			client_id: None,
			role_id: None,
			user_id: None
		}
	}
}

impl Form {
	pub fn from_id(_id: Option<i32>) -> Self {
		let mut form = Form::default();
		form.id = _id;
		form
	}
}

impl IndexDocument for Form {
	fn mapping(self: &Self) -> serde_json::Value {
		json!({
			"mappings" : {
				"properties" : {
					"uuid" : { "type" : "text" },
					"id" : { "type" : "integer" },
					"file_name" : { "type" : "text" },
					"name" : { "type" : "text" },
					"description" : { "type" : "text" },
					"help" : { "type" : "text" }
				}
			}
		})
	}

	fn data(self: &Self) -> serde_json::Value {
		json!(self)
	}

	fn id(self: &Self) -> String {
		self.id.unwrap().to_string()
	}

	fn index_name(self: &Self) -> String {
		match &self.index_value {
			Some(value) => value.to_string(),
			None => "forms".to_string(),
		}
	}

	fn find(self: &Self, _search_value: String) -> serde_json::Value {
		let mut query = "*".to_owned();
		query.push_str(&_search_value.to_owned());
		query.push_str(&"*".to_owned());

		json!({
			"query": {
				"query_string": {
				"query": query
				}
			}
		})
	}
}

pub async fn form_from_id(_id: Option<i32>, _language: Option<&String>, _client_id: Option<&String>, _role_id: Option<&String>, _user_id: Option<&String>) -> Result<Form, String> {
	if _id.is_none() || _id.map(|id| id <= 0).unwrap_or(false) {
		return Err(Error::new(ErrorKind::InvalidData.into(), "Form Identifier is Mandatory").to_string());
	}
	let mut _document = Form::from_id(_id);

	let _index_name = match get_index_name(_language, _client_id, _role_id, _user_id).await {
		Ok(index_name) => index_name,
		Err(error) => {
			log::error!("Index name error: {:?}", error.to_string());
			return Err(error.to_string())
		}
	};
	log::info!("Index to search {:}", _index_name);

	_document.index_value = Some(_index_name);
	let _form_document: &dyn IndexDocument = &_document;
	match get_by_id(_form_document).await {
		Ok(value) => {
			let form: Form = serde_json::from_value(value).unwrap();
			log::info!("Finded Value: {:?}", form.id);
			// Ok(FormResponse {
			// 	form: Some(form)
			// })
			Ok(
				form
			)
		},
		Err(error) => {
			log::error!("{}", error);
			Err(error)
		},
	}
}

async fn get_index_name(_language: Option<&String>, _client_id: Option<&String>, _role_id: Option<&String>, _user_id: Option<&String>) -> Result<String, std::io::Error> {
	//  Validate
	if _language.is_none() {
		return Err(Error::new(ErrorKind::InvalidData.into(), "Language is Mandatory"));
	}
	if _client_id.is_none() {
		return Err(Error::new(ErrorKind::InvalidData.into(), "Client is Mandatory"));
	}

	let _index: String = "form".to_string();

	let _client_index = client_index(_index.to_owned(), _language, _client_id);

	//  Find index
	match exists_index(_client_index.to_owned()).await {
		Ok(_) => {
			log::info!("Find with client index `{:}`", _client_index);
			Ok(_client_index)
		},
		Err(error) => {
			log::error!("No client index `{:}`", _client_index);
			return Err(Error::new(ErrorKind::InvalidData.into(), error))
		}
	}
}

pub async fn forms(_language: Option<&String>, _client_id: Option<&String>, _role_id: Option<&String>, _user_id: Option<&String>, _search_value: Option<&String>) -> Result<FormsListResponse, std::io::Error> {
	let _search_value = match _search_value {
		Some(value) => value.clone(),
		None => "".to_owned()
	};

	//  Find index
	let _index_name = match get_index_name(_language, _client_id, _role_id, _user_id).await {
		Ok(index_name) => index_name,
		Err(error) => {
			log::error!("Index name error: {:?}", error.to_string());
			return Err(Error::new(ErrorKind::InvalidData.into(), error))
		}
	};
	log::info!("Index to search {:}", _index_name);

	let mut _document = Form::default();
	_document.index_value = Some(_index_name);
	let _forms_document: &dyn IndexDocument = &_document;
	match find(_forms_document, _search_value, 0, 10).await {
		Ok(values) => {
			let mut forms_list: Vec<Form> = vec![];
			for value in values {
				let form: Form = serde_json::from_value(value).unwrap();
				forms_list.push(form.to_owned());
			}
			Ok(FormsListResponse {
				forms: Some(forms_list)
			})
		},
		Err(error) => Err(Error::new(ErrorKind::InvalidData.into(), error))
	}
}
