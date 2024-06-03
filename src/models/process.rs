use serde::{Deserialize, Serialize};
use salvo::prelude::*;
use serde_json::json;
use std::{io::ErrorKind, io::Error};

use crate::controller::opensearch::{IndexDocument, get_by_id, find, exists_index};

use super::{client_index, user_index, role_index};

#[derive(Deserialize, Extractible, Debug, Clone)]
#[salvo(extract(default_source(from = "body")))]
pub struct ProcessDocument {
    pub document: Option<Process>
}

#[derive(Serialize, Debug, Clone)]
pub struct ProcessResponse {
    pub process: Option<Process>
}

#[derive(Serialize, Debug, Clone)]
pub struct ProcessListResponse {
    pub processes: Option<Vec<Process>>
}

impl Default for ProcessResponse {
    fn default() -> Self {
        ProcessResponse {
            process: None
        }
    }
}

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct DependendField {
    pub uuid: Option<String>,
    pub id: Option<i32>,
    pub column_name: Option<String>,
    pub parent_id: Option<i32>,
    pub parent_uuid: Option<String>,
    pub parent_name: Option<String>
}

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct DictionaryEntity {
	pub id: Option<i32>,
	pub uuid: Option<String>,
	pub name: Option<String>,
	pub description: Option<String>,
	pub help: Option<String>
}

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct Process {
    pub uuid: Option<String>,
    pub id: Option<i32>,
	pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub help: Option<String>,
	pub is_active: Option<bool>,
    pub show_help: Option<String>,
	//	Report
    pub is_report: Option<bool>,
    pub report_view_id: Option<i32>,
    pub print_format_id: Option<i32>,
	//	Linked
	pub browser_id: Option<i32>,
	pub browse: Option<DictionaryEntity>,
	pub form_id: Option<i32>,
	pub form: Option<DictionaryEntity>,
	pub workflow_id: Option<i32>,
	pub workflow: Option<DictionaryEntity>,
	//	Index
    pub index_value: Option<String>,
    pub language: Option<String>,
    pub client_id: Option<i32>,
    pub role_id: Option<i32>,
    pub user_id: Option<i32>,
	//	Parameters
    pub has_parameters: Option<bool>,
    pub parameters: Option<Vec<ProcessParameters>>
}

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct Reference {
	pub context_column_names: Option<Vec<String>>
}

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct ProcessParameters {
    pub uuid: Option<String>,
    pub id: Option<i32>,
	pub column_name: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
	pub is_active: Option<bool>,
    pub help: Option<String>,
	pub display_type: Option<i32>,
	//	Value Properties
	pub is_range: Option<bool>,
    pub default_value: Option<String>,
    pub default_value_to: Option<String>,
	pub field_length: Option<i32>,
    pub value_format: Option<String>,
    pub min_value: Option<String>,
    pub max_value: Option<String>,
	//	Display Properties
	pub display_logic: Option<String>,
    pub sequence: Option<i32>,
    pub is_displayed_as_panel: Option<String>,
	//	Mandatory Properties
	pub is_mandatory: Option<bool>,
	//	Editable Properties
	pub read_only_logic: Option<String>,
	pub is_info_only: Option<bool>,
	// External Info
    pub context_column_names: Option<Vec<String>>,
	pub reference: Option<Reference>,
    pub dependent_fields: Option<Vec<DependendField>>
}

impl Default for Process {
    fn default() -> Self {
        Self { 
            uuid: None, 
            id: None, 
			code: None,
            name: None, 
            description: None, 
            help: None, 
			is_active: None,
			show_help: None,
			//	Report
			is_report: None,
			print_format_id: None,
			report_view_id: None,
			//	Linked
			browser_id: None,
            browse: None,
			form_id: None,
			form: None, 
			workflow_id: None,
			workflow: None,
			//	Index
            index_value: None,
            language: None,
			client_id: None,
            role_id: None,
            user_id: None,
			// Parameters
            parameters: None,
            has_parameters: None
        }
    }
}

impl Process {
    pub fn from_id(_id: Option<i32>) -> Self {
        let mut process = Process::default();
        process.id = _id;
        process
    }
}

impl IndexDocument for Process {
    fn mapping(self: &Self) -> serde_json::Value {
        json!({
            "mappings" : {
                "properties" : {
                    "uuid" : { "type" : "text" },
                    "id" : { "type" : "integer" },
                    "code" : { "type" : "text" },
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
            None => "process".to_string(),
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

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct Form {
    pub uuid: Option<String>,
    pub id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub help: Option<String>,
}

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct Browse {
    pub uuid: Option<String>,
    pub id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub help: Option<String>,
}

#[derive(Deserialize, Serialize, Extractible, Debug, Clone)]
pub struct Workflow {
    pub uuid: Option<String>,
    pub id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub help: Option<String>,
}

pub async fn process_from_id(_id: Option<i32>, _language: Option<&String>, _client_id: Option<&String>, _role_id: Option<&String>, _user_id: Option<&String>) -> Result<Process, String> {
	if _id.is_none() || _id.map(|id| id <= 0).unwrap_or(false) {
		return Err(Error::new(ErrorKind::InvalidData.into(), "Process/Report Identifier is Mandatory").to_string());
	}
    let mut _document = Process::from_id(_id);

	let _index_name = match get_index_name(_language, _client_id, _role_id, _user_id).await {
		Ok(index_name) => index_name,
		Err(error) => {
			log::error!("Index name error: {:?}", error.to_string());
			return Err(error.to_string())
		}
	};
	log::info!("Index to search {:}", _index_name);

    _document.index_value = Some(_index_name);
    let _process_document: &dyn IndexDocument = &_document;
    match get_by_id(_process_document).await {
        Ok(value) => {
			let mut process: Process = serde_json::from_value(value).unwrap();
            log::info!("Finded Value: {:?}", process.id);

			// sort process parameter by sequence
			if let Some(ref mut parameters) = process.parameters {
				parameters.sort_by_key(|parameter| parameter.sequence.clone().unwrap_or(0));
			}

            Ok(
                process
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
    if _role_id.is_none() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Role is Mandatory"));
    }

	let _index: String = "process".to_string();

	let _user_index = user_index(_index.to_owned(), _language, _client_id, _role_id, _user_id);
    let _role_index = role_index(_index.to_owned(), _language, _client_id, _role_id);
	let _client_index = client_index(_index.to_owned(), _language, _client_id);

    //  Find index
    match exists_index(_user_index.to_owned()).await {
		Ok(_) => {
			log::info!("Find with user index `{:}`", _user_index);
			Ok(_user_index)
		},
        Err(_) => {
			log::warn!("No user index `{:}`", _user_index);
            match exists_index(_role_index.to_owned()).await {
                Ok(_) => {
					log::info!("Find with role index `{:}`", _role_index);
					Ok(_role_index)
				},
				Err(error) => {
					log::warn!("No role index `{:}`", _role_index);
					match exists_index(_client_index.to_owned()).await {
						Ok(_) => {
							log::info!("Find with client index `{:}`", _client_index);
							Ok(_client_index)
						},
						Err(_) => {
							log::error!("No client index `{:}`", _client_index);
							return Err(Error::new(ErrorKind::InvalidData.into(), error))
						}
					}
                }
            }
        }
    }
}

pub async fn processes(_language: Option<&String>, _client_id: Option<&String>, _role_id: Option<&String>, _user_id: Option<&String>, _search_value: Option<&String>) -> Result<ProcessListResponse, std::io::Error> {
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

    let mut _document = Process::default();
    _document.index_value = Some(_index_name);
    let _process_document: &dyn IndexDocument = &_document;
    match find(_process_document, _search_value, 0, 10).await {
        Ok(values) => {
            let mut processes_list: Vec<Process> = vec![];
            for value in values {
				let mut process: Process = serde_json::from_value(value).unwrap();
				// sort process parameter by sequence
				if let Some(ref mut parameters) = process.parameters {
					parameters.sort_by_key(|parameter| parameter.sequence.clone().unwrap_or(0));
				}
                processes_list.push(process.to_owned());
            }

            Ok(ProcessListResponse {
                processes: Some(processes_list)
            })
        },
        Err(error) => Err(Error::new(ErrorKind::InvalidData.into(), error))
    }
}
