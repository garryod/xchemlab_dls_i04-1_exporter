const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";

const MODELS_PATH: &str = "src/";

const TABLES: &'static [&str] = &[
    "Shipping",
    "LabContact",
    "Person",
    "Proposal",
    "Laboratory",
    "Dewar",
    "BLSession",
    "BeamLineSetup",
    "BeamCalendar",
    "Detector",
];

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let database_url_var = std::env::var(DATABASE_URL_ENV_VAR).expect(&format!(
                "The {} environment variable must point to an instance of ISPyB.",
                DATABASE_URL_ENV_VAR
            ));
            let database_url = url::Url::parse(&database_url_var).expect(&format!(
                "The {} ({}) could not be parsed as a URL.",
                DATABASE_URL_ENV_VAR, database_url_var
            ));

            let database_name = database_url
                .path_segments()
                .map(|mut segments| segments.next())
                .flatten()
                .expect(&format!(
                    "The {} ({}) does not contain a database name.",
                    DATABASE_URL_ENV_VAR, database_url_var
                ));

            let connection = sqlx::Pool::<sqlx::MySql>::connect(database_url.as_str())
                .await
                .expect(&format!(
                    "Could not connect to {} ({}).",
                    DATABASE_URL_ENV_VAR, database_url_var
                ));

            let schema_discovery =
                sea_schema::mysql::discovery::SchemaDiscovery::new(connection, database_name);
            let schema = schema_discovery.discover().await;
            let table_statements = schema
                .tables
                .into_iter()
                .filter(|table| TABLES.contains(&table.info.name.as_str()))
                .map(|table| table.write())
                .collect();

            let writer_context = sea_orm_codegen::EntityWriterContext::new(
                false,
                sea_orm_codegen::WithSerde::None,
                true,
                sea_orm_codegen::DateTimeCrate::Chrono,
                None,
                true,
                false,
                false,
                vec![],
                vec![],
            );

            let output = sea_orm_codegen::EntityTransformer::transform(table_statements)
                .unwrap()
                .generate(&writer_context);

            let models_path = std::path::Path::new(MODELS_PATH);
            std::fs::create_dir_all(models_path).expect(&format!(
                "Could not create directory at {}",
                models_path.to_str().unwrap()
            ));

            output.files.iter().for_each(|output| {
                let file_path = models_path.join(output.name.clone());
                let mut file = std::fs::File::create(&file_path).expect(&format!(
                    "Could not create file at {}",
                    file_path.to_str().unwrap()
                ));
                std::io::Write::write_all(&mut file, output.content.as_bytes()).expect(&format!(
                    "Could not write to file at {}",
                    file_path.to_str().unwrap()
                ));
            });

            output.files.iter().for_each(|output| {
                let file_path = models_path.join(output.name.clone());
                let rustfmt_exit = std::process::Command::new("rustfmt")
                    .arg(file_path.to_str().unwrap())
                    .status()
                    .expect(&format!(
                        "Could not run 'rustfmt' on {}",
                        file_path.to_str().unwrap()
                    ));
                if !rustfmt_exit.success() {
                    panic!("Failed to format {}", file_path.to_str().unwrap())
                };
            });
        })
}
