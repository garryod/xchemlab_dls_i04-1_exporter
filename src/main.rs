#[forbid(unsafe_code)]
#[warn(missing_docs)]
mod api;

use api::{RootMutation, RootQuery, RootSchema, RootSubscription};
use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Extension, Router, Server,
};
use sea_orm::Database;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

async fn graphql_handler(schema: Extension<RootSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("");
    let database = Database::connect(&database_url)
        .await
        .expect("The DATABASE_URL environment variable must point to a live instance of ISPyB");

    let schema = Schema::build(
        RootQuery::default(),
        RootMutation::default(),
        RootSubscription::default(),
    )
    .data(database)
    .finish();

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema));

    let socket_addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 13245));
    println!("GraphiQL IDE: {}", socket_addr);
    Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
