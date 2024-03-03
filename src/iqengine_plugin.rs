use futuresdr::{
    anyhow::{Ok, Result},
    log::debug,
};

use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::Html,
    routing::{get, options, post},
    Json, Router,
};
use iqengine_plugin::server::{
    FunctionParameters, FunctionPostRequest, FunctionPostResponse, IQFunction,
};
use simple_logger::SimpleLogger;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use fsdr_cli::iqengine_userdef::{UserDefinedFunctionParams, USER_DEFINED_FUNCTION};

#[tokio::main]
pub async fn start_iqengine_daemon(_filename: Option<&str>) -> Result<()> {
    SimpleLogger::new().init().unwrap();

    // initialize tracing
    //tracing_subscriber::fmt::init();

    // let cors = CorsLayer::new()
    //     .allow_origin(Any)
    //     .allow_headers(Any)
    //     // .allow_credentials(true)
    //     .allow_methods(vec![Method::GET, Method::POST]);
    let cors = CorsLayer::very_permissive();

    // build our application with a route
    let app = Router::new()
        .route("/", get(get_index))
        .route("/plugins", get(get_functions_list))
        .route("/plugins/", get(get_functions_list))
        .route("/plugins/:functionname", options(options_function))
        .route("/plugins/userdef", get(get_userdef_params))
        .route("/plugins/userdef", post(post_userdef))
        .layer(ServiceBuilder::new().layer(cors))
        .layer(DefaultBodyLimit::disable());

    let addr = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    println!("listening on {}", 8000);
    axum::serve(addr, app).await.expect("msg");
    Ok(())
}

async fn options_function() -> (StatusCode, Json<String>) {
    (StatusCode::OK, Json("preflight ok".to_string()))
}

async fn get_index() -> (StatusCode, Html<&'static str>) {
    debug!("Get root");
    (
        StatusCode::OK,
        Html("Welcome to the IQEngine plugin written in Rust and FutureSDR."),
    )
}

// Return list of IQEngine functions
async fn get_functions_list() -> (StatusCode, Json<Vec<&'static str>>) {
    debug!("Get function list");
    let functions_list = vec!["userdef"];
    (StatusCode::OK, Json(functions_list))
}

// Describe the parameters for the fm-receiver
async fn get_userdef_params() -> (StatusCode, Json<FunctionParameters>) {
    debug!("Get parameters of user defined function");
    let custom_params = USER_DEFINED_FUNCTION.parameters();
    (StatusCode::OK, Json(custom_params))
}

// Apply the fm-receiver
#[axum::debug_handler]
async fn post_userdef(
    Json(req): Json<FunctionPostRequest<UserDefinedFunctionParams>>,
) -> (StatusCode, Json<FunctionPostResponse>) {
    debug!("Request for user defined function");
    let res = USER_DEFINED_FUNCTION.apply(req).await;
    if let std::prelude::v1::Result::Ok(res) = res {
        return (StatusCode::OK, Json(res));
    }
    let mut resp = FunctionPostResponse::new();
    let details = res.unwrap_err().to_string();
    resp.details = Some(details.clone());
    debug!("Bad Request for user defined function {details}");
    (StatusCode::BAD_REQUEST, Json(resp))
}
