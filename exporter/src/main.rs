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
use std::{
    fs::File,
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
};

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

async fn serve(router: Router, port: u16) {
    let socket_addr: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));
    println!("GraphiQL IDE: {}", socket_addr);
    Server::bind(&socket_addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
enum Cli {
    /// Starts a webserver serving the GraphQL API
    Serve(ServeArgs),
    /// Prints the GraphQL API to stdout
    Schema(SchemaArgs),
}

#[derive(Debug, Parser)]
struct ServeArgs {
    /// The port number to serve on.
    #[arg(short, long, default_value_t = 80)]
    port: u16,
}

#[derive(Debug, Parser)]
struct SchemaArgs {
    /// The file path to write the schema to. If not supplied the schema will be printed to stdout.
    #[arg(short, long)]
    path: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args {
        Cli::Serve(args) => {
            let database = setup_database().await;
            let schema = setup_api(database).await;
            let router = setup_router(schema).await;
            serve(router, args.port).await;
        }
        Cli::Schema(args) => {
            let database = setup_database().await;
            let schema = setup_api(database).await;
            let schema_string = schema.sdl();
            if let Some(path) = args.path {
                let mut file = File::create(path).unwrap();
                file.write_all(schema_string.as_bytes()).unwrap();
            } else {
                println!("{}", schema_string);
            }
        }
    }
}
