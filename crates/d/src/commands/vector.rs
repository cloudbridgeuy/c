use std::collections::HashMap;
use std::ops::RangeInclusive;

use clap::{Parser, Subcommand};
use color_eyre::eyre::eyre;
use color_eyre::eyre::{bail, Result};
use uuid::Uuid;

const DIMENSION: usize = 1536;
const DISTANCE: crate::similarity::Distance = crate::similarity::Distance::Cosine;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Creates a new vector db collection
    #[clap(name = "create")]
    Create(CreateOptions),
    /// Deletes a vector db collection
    #[clap(name = "delete")]
    Delete(GetOptions),
    /// Gets the list of available collections
    #[clap(name = "list")]
    List,
    /// Gets a vector db collection
    #[clap(name = "get")]
    Get(GetOptions),
    /// Queries a vector db collection
    #[clap(name = "query")]
    Query(QueryOptions),
    /// Inserts an embedding into a collection
    #[clap(name = "insert")]
    Insert(InsertOptions),
}

#[derive(Default, Clone, Parser, Debug)]
pub struct CreateOptions {
    /// Name of the collection
    name: String,
    /// Collection dimension
    #[arg(long, default_value = "1536")]
    dimension: Option<usize>,
    /// Collection distance function to use
    #[clap(long, value_enum, default_value = "cosine")]
    distance: Option<crate::similarity::Distance>,
}

#[derive(Default, Clone, Parser, Debug)]
pub struct GetOptions {
    /// Name of the collection
    name: String,
}

/// The range of values for the `score` option which goes from 0 to 1.
const SCORE_RANGE: RangeInclusive<f32> = 0.0..=1.0;

/// Validates an input that's between 0 and 1
fn parse_score(s: &str) -> std::result::Result<f32, String> {
    let value = s
        .parse::<f32>()
        .map_err(|_| format!("`{s}` must be a number between {} and {}", 0.0, 1.0,))?;
    if !SCORE_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            0.0, 1.0,
        ));
    }

    Ok(value)
}

#[derive(Default, Clone, Parser, Debug)]
pub struct QueryOptions {
    /// Name of the collection
    name: String,
    /// The query string
    #[arg(value_delimiter = ',', allow_hyphen_values = true)]
    query: Vec<f32>,
    /// The number of results to return
    #[arg(long, default_value = "1")]
    k: Option<usize>,
    /// Filter values that score less than this value.
    #[clap(long, value_parser = parse_score)]
    score: Option<f32>,
}

/// Parse a single key-value pair
fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Default, Clone, Parser, Debug)]
pub struct InsertOptions {
    /// Name of the collection
    name: String,
    /// Embedding Vector
    #[arg(value_delimiter = ',', allow_hyphen_values = true)]
    vector: Vec<f32>,
    /// Embedding Id
    #[arg(long)]
    id: Option<String>,
    /// Embedding Metadata
    #[arg(long, value_parser = parse_key_val::<String, String>, value_delimiter = ',')]
    metadata: Option<Vec<(String, String)>>,
}

#[derive(Debug, Parser)]
#[command(name = "vector")]
#[command(about = "Manage vector collections")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Runs the `vector` subcommand
pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Create(options)) => create(options).await?,
        Some(Commands::Delete(options)) => delete(options).await?,
        Some(Commands::Get(options)) => get(options).await?,
        Some(Commands::Query(options)) => query(options).await?,
        Some(Commands::Insert(options)) => insert(options).await?,
        Some(Commands::List) => list().await?,
        None => {
            bail!("No subcommand provided. Use `d help` to see the list of available subcommands.")
        }
    }

    Ok(())
}

pub async fn insert(options: InsertOptions) -> Result<()> {
    let mut db = crate::vector::from_store()?;
    let mut metadata: HashMap<String, String> = HashMap::new();

    if let Some(tuples) = options.metadata {
        for (key, value) in tuples {
            metadata.insert(key, value);
        }
    }

    let embedding = crate::vector::Embedding {
        id: options.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        vector: options.vector,
        metadata: Some(metadata),
    };

    db.insert_into_collection(&options.name, embedding)?;

    println!("Inserted into {} collection", options.name);

    Ok(())
}

pub async fn query(options: QueryOptions) -> Result<()> {
    let db = crate::vector::from_store()?;

    match db.get_collection(&options.name) {
        Some(collection) => {
            if collection.dimension != options.query.len() {
                return Err(eyre!(
                    "Collection {} does not have the same dimension of the query",
                    options.name
                ));
            }

            let instant = std::time::Instant::now();
            let results = collection.get_similarity(&options.query, options.k.unwrap_or(1));

            // Filter those results whose value is less than `options.score` it its `Some`
            let results = match options.score {
                Some(score) => results.into_iter().filter(|x| x.score >= score).collect(),
                None => results,
            };

            log::info!("Query to {} took {:?}", &options.name, instant.elapsed());

            println!("{:#?}", results);

            Ok(())
        }
        None => Err(eyre!("Collection {} does not exist", options.name)),
    }
}

pub async fn list() -> Result<()> {
    let db = crate::vector::from_store()?;

    println!("{:#?}", db.list_collections());

    Ok(())
}

pub async fn get(options: GetOptions) -> Result<()> {
    let db = crate::vector::from_store()?;

    let collection = db.get_collection(&options.name).unwrap();

    println!("{:#?}", collection);

    Ok(())
}

pub async fn delete(options: GetOptions) -> Result<()> {
    let mut db = crate::vector::from_store()?;

    db.delete_collection(&options.name)?;

    println!("Deleted {} collection", options.name);

    Ok(())
}

pub async fn create(options: CreateOptions) -> Result<()> {
    let mut db = crate::vector::from_store()?;

    let collection = db.create_collection(
        options.name,
        options.dimension.unwrap_or(DIMENSION),
        options.distance.unwrap_or(DISTANCE),
    )?;

    println!("{:#?}", collection);

    Ok(())
}
