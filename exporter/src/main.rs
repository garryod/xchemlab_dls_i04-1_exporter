#[forbid(unsafe_code)]
#[warn(missing_docs)]
mod api;
mod broker;

use self::api::{RootMutation, RootQuery, RootSchema, RootSubscription};
use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router, Server,
};
use clap::Parser;
use sea_orm::{Database, DatabaseConnection};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

async fn setup_database() -> DatabaseConnection {
    let database_url = std::env::var("DATABASE_URL").expect("");
    Database::connect(&database_url)
        .await
        .expect("The DATABASE_URL environment variable must point to a live instance of ISPyB")
}

async fn setup_api(
    database: DatabaseConnection,
) -> Schema<RootQuery, RootMutation, RootSubscription> {
    Schema::build(
        RootQuery::default(),
        RootMutation::default(),
        RootSubscription::default(),
    )
    .data(database)
    .finish()
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/")
            .subscription_endpoint("/ws")
            .finish(),
    )
}

async fn graphql_handler(schema: Extension<RootSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn setup_router<Q, M, S>(schema: Schema<Q, M, S>) -> Router
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
    S: async_graphql::SubscriptionType + 'static,
{
    Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema))
}

async fn serve(router: Router) {
    let socket_addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 13245));
    println!("GraphiQL IDE: {}", socket_addr);
    Server::bind(&socket_addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
enum Cli {
    /// Starts a webserver serving the GraphQL API
    Serve,
    /// Prints the GraphQL API to stdout
    Schema,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args {
        Cli::Serve => {
            let database = setup_database().await;
            let schema = setup_api(database).await;
            let router = setup_router(schema).await;
            serve(router).await;
        }
        Cli::Schema => {
            let database = setup_database().await;
            let schema = setup_api(database).await;
            let schema_string = schema.sdl();
            println!("{}", schema_string);
        }
    }
}
