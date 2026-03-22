use std::env;

use salvo::{Request, Response, Router, cors::Cors, handler, http::{Method, StatusCode, header}, writing::Json};
use serde::Serialize;

use crate::models::{browser::{browser_from_id, browsers}, form::{form_from_id, forms}, menu::allowed_menu, process::{process_from_id, processes}, window::{window_from_id, windows}};


pub fn routes() -> Router {
	// TODO: Add support to allow requests from multiple origin
	let allowed_origin: String = match env::var("ALLOWED_ORIGIN") {
		Ok(value) => value,
		Err(_) => {
			log::warn!("Variable `ALLOWED_ORIGIN` Not found from enviroment");
			"*".to_owned()
		}.to_owned()
	};

	let allow_methods: Vec<Method> = vec![
		Method::OPTIONS,
		Method::GET
	];
	let allow_headers: Vec<header::HeaderName> = vec![
		header::ACCESS_CONTROL_REQUEST_METHOD,
		header::ACCESS_CONTROL_REQUEST_HEADERS,
		header::AUTHORIZATION
	];
	// Send Device Info
	let cors_handler = Cors::new()
		.allow_origin(&allowed_origin.to_owned())
		.allow_methods(allow_methods)
		.allow_headers(allow_headers)
		.into_handler()
	;

	let router: Router = Router::new()
		.hoop(cors_handler)
		// /	root path
		.options(options_response)
		.get(get_system_info)
		.push(
			// /api
			Router::with_path("api")
				.options(options_response)
				.get(get_system_info)
				.push(
					// /api/system-info
					Router::with_path("system-info")
						.options(options_response)
						.get(get_system_info),
				)
				.push(
					// /api/security/menus
					Router::with_path("security/menus")
						.options(options_response)
						.get(get_allowed_menu),
				)
				.push(
					// /api/dictionary
					Router::with_path("dictionary")
						.push(
							// /api/dictionary/system-info
							Router::with_path("system-info")
								.options(options_response)
								.get(get_system_info)
						)
						.push(
							// /api/dictionary/browsers/
							Router::with_path("browsers")
								.options(options_response)
								.get(get_browsers)
								.push(
									// /api/dictionary/browsers/:id
									Router::with_path("{id}")
										.options(options_response)
										.get(get_browsers)
								)
						)
						.push(
							// /api/dictionary/forms/
							Router::with_path("forms")
								.options(options_response)
								.get(get_forms)
								.push(
									// /api/dictionary/forms/:id
									Router::with_path("{id}")
										.options(options_response)
										.get(get_forms)
								)
						)
						.push(
							// /api/dictionary/processes
							Router::with_path("processes")
								.options(options_response)
								.get(get_processes)
								.push(
									// /api/dictionary/processes/:id
									Router::with_path("{id}")
										.options(options_response)
										.get(get_processes)
								)
						)
						.push(
							// /api/dictionary/windows/
							Router::with_path("windows")
								.options(options_response)
								.get(get_windows)
								.push(
									// /api/dictionary/windows/:id
									Router::with_path("{id}")
										.options(options_response)
										.get(get_windows)
								)
						)
				)
		)
	;

	log::info!("{:#?}", router);
	router
}


#[handler]
async fn options_response<'a>(_req: &mut Request, _res: &mut Response) {
	_res.status_code(StatusCode::NO_CONTENT);
}

#[derive(Serialize)]
struct ErrorResponse {
	status: u16,
	message: String
}


#[derive(Serialize)]
struct SystemInfoResponse {
	version: String,
	is_kafka_enabled: bool,
	kafka_queues: String,
}

#[handler]
async fn get_system_info<'a>(_req: &mut Request, _res: &mut Response) {
	let version: String = match env::var("VERSION") {
		Ok(value) => value,
		Err(_) => {
			log::warn!("Variable `VERSION` Not found from enviroment, as default `1.0.0-dev`");
			"1.0.0-dev".to_owned()
		}.to_owned()
	};

	// Kafka Queue
	let kafka_enabled: String = match env::var("KAFKA_ENABLED") {
		Ok(value) => value,
		Err(_) => {
			log::warn!("Variable `KAFKA_ENABLED` Not found from enviroment, as default Y");
			"Y".to_owned()
		}.to_owned()
	};
	let kafka_queues: String = match env::var("KAFKA_QUEUES") {
		Ok(value) => value.clone(),
		Err(_) => {
			log::warn!("Variable `KAFKA_QUEUES` Not found from enviroment, loaded with `default` value");
			"browser form process window menu_item menu_tree role".to_owned()
		}.to_owned()
	};

	let system_info_response: SystemInfoResponse = SystemInfoResponse {
		version: version.to_string(),
		is_kafka_enabled: kafka_enabled.trim().eq("Y"),
		kafka_queues: kafka_queues
	};

	_res.status_code(StatusCode::OK)
		.render(
			Json(system_info_response)
		)
	;
}

#[handler]
async fn get_forms<'a>(_req: &mut Request, _res: &mut Response) {
	let mut _id: Option<String> = _req.param::<String>("id");
	if _id.is_none() {
		// fill with query url
		_id = _req.queries().get("id").map(|s| s.to_owned());
	}
	log::debug!("Get by ID: {:?}", _id);

	let _language: Option<&String> = _req.queries().get("language");
	let _dictionary_code: Option<&String> = _req.queries().get("dictionary_code");
	let _search_value: Option<&String> = _req.queries().get("search_value");
	if _id.is_some() {
		match form_from_id(_id, _language, _dictionary_code).await {
			Ok(form) => _res.render(Json(form)),
			Err(error) => {
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
				_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
			}
		}
	} else {
		let _search_value: Option<&String> = _req.queries().get("search_value");
		match forms(_language, _search_value, _dictionary_code).await {
			Ok(forms_list) => {
				_res.render(Json(forms_list));
			},
			Err(error) => {
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
				_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
			}
		}
	}
}

#[handler]
async fn get_allowed_menu<'a>(_req: &mut Request, _res: &mut Response) {
	let _language: Option<&String> = _req.queries().get("language");
	let _client_id: Option<&String> = _req.queries().get("client_id");
	let _role_id: Option<&String> = _req.queries().get("role_id");
	let _dictionary_code: Option<&String> = _req.queries().get("dictionary_code");
	match allowed_menu(_language, _client_id, _role_id, _dictionary_code).await {
		Ok(menu) => _res.render(Json(menu)),
		Err(error) => {
			let error_response: ErrorResponse = ErrorResponse {
				status: StatusCode::INTERNAL_SERVER_ERROR.into(),
				message: error.to_string()
			};
			_res.render(
				Json(error_response)
			);
			_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
		}
	}
}

#[handler]
async fn get_processes<'a>(_req: &mut Request, _res: &mut Response) {
	let mut _id: Option<String> = _req.param::<String>("id");
	if _id.is_none() {
		// fill with query url
		_id = _req.queries().get("id").map(|s| s.to_owned());
	}
	log::debug!("Get by ID: {:?}", _id);

	let _language: Option<&String> = _req.queries().get("language");
	let _dictionary_code: Option<&String> = _req.queries().get("dictionary_code");
	let _search_value: Option<&String> = _req.queries().get("search_value");
	if _id.is_some() {
		match process_from_id(_id, _language, _dictionary_code).await {
			Ok(process) => _res.render(Json(process)),
			Err(error) => {
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
				_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
			}
		}
	} else {
		match processes(_language, _search_value, _dictionary_code).await {
			Ok(processes_list) => {
				_res.render(Json(processes_list));
			},
			Err(error) => {
				let error_response = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
				_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
			}
		}
	}
}

#[handler]
async fn get_browsers<'a>(_req: &mut Request, _res: &mut Response) {
	let mut _id: Option<String> = _req.param::<String>("id");
	if _id.is_none() {
		// fill with query url
		_id = _req.queries().get("id").map(|s| s.to_owned());
	}
	log::debug!("Get by ID: {:?}", _id);

	let _language: Option<&String> = _req.queries().get("language");
	let _dictionary_code: Option<&String> = _req.queries().get("dictionary_code");
	let _search_value: Option<&String> = _req.queries().get("search_value");
	if _id.is_some() {
		match browser_from_id(_id, _language, _dictionary_code).await {
			Ok(browser) => _res.render(Json(browser)),
			Err(error) => {
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
				_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
			}
		}
	} else {
		match browsers(_language, _search_value, _dictionary_code).await {
			Ok(browsers_list) => {
				_res.render(Json(browsers_list));
			},
			Err(error) => {
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
				_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
			}
		}
	}
}

#[handler]
async fn get_windows<'a>(_req: &mut Request, _res: &mut Response) {
	let mut _id: Option<String> = _req.param::<String>("id");
	if _id.is_none() {
		_id = _req.queries().get("id").map(|s| s.to_owned());
	}
	log::debug!("Get by ID: {:?}", _id);

	let _language: Option<&String> = _req.queries().get("language");
	let _dictionary_code: Option<&String> = _req.queries().get("dictionary_code");
	let _search_value: Option<&String> = _req.queries().get("search_value");
	if _id.is_some() {
		match window_from_id(_id, _language, _dictionary_code).await {
			Ok(window) => _res.render(Json(window)),
			Err(error) => {
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
				_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
			}
		}
	} else {
		match windows(_language, _search_value, _dictionary_code).await {
			Ok(windows_list) => {
				_res.render(Json(windows_list));
			},
			Err(error) => {
				let error_response: ErrorResponse = ErrorResponse {
					status: StatusCode::INTERNAL_SERVER_ERROR.into(),
					message: error.to_string()
				};
				_res.render(
					Json(error_response)
				);
				_res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
			}
		}
	}
}
