use std::sync::Arc;

use askama::Template;
use axum::{extract::Path, http::StatusCode, routing::get, Router};
use num::{BigUint, Num};
use tokio::task::JoinSet;

mod nerds;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/:n", get(handle_int));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "int.html")]
struct IntTemplate {
    n: Arc<BigUint>,
    facts: Vec<String>,
}

async fn handle_int(Path(param): Path<String>) -> Result<IntTemplate, (StatusCode, String)> {
    let Ok(n) = BigUint::from_str_radix(&param, 10) else {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Error: {param:?} could not be parsed as an unsigned integer."),
        ));
    };
    let n = Arc::new(n);

    // Could/should we use scoped threads here?
    let mut tasks = JoinSet::new();
    for nerd in nerds::NERDS {
        let n = Arc::clone(&n);
        tasks.spawn_blocking(move || nerd(n));
    }

    let mut facts = Vec::new();
    while let Some(res) = tasks.join_next().await {
        if let Ok(Some(fact)) = res {
            facts.push(fact);
        }
    }
    
    Ok(IntTemplate { n, facts })
}
