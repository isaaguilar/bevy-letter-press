use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client as DynamoClient, Error as DynamoError};
use lambda_http::{Body, Error, Request, Response, run, service_fn};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize, Debug)]
struct ScoreEntry {
    name: String,
    score: u32,
    level: usize,
}

impl ScoreEntry {
    fn add(name: impl Into<String>, score: u32, level: usize) -> Self {
        Self {
            level: level,
            name: name.into(),
            score: score,
        }
    }
}

#[derive(Serialize, Debug)]
struct Output {
    leaderboard: Vec<ScoreEntry>,
}

async fn handler(event: Request) -> Result<Response<Body>, Error> {
    let input: ScoreEntry = match serde_json::from_slice(event.body()) {
        Ok(data) => data,
        Err(_) => {
            return Ok(Response::builder()
                .status(400)
                .body("Invalid input".into())
                .unwrap());
        }
    };

    let table_name = env::var("LEADERBOARD_TABLE").unwrap_or_else(|_| "default_table".to_string());

    let config = aws_config::load_from_env().await;
    let client = DynamoClient::new(&config);

    if !input.name.is_empty() {
        // Insert score into DynamoDB
        let _ = client
            .put_item()
            .table_name(&table_name)
            .item("name", AttributeValue::S(input.name.clone()))
            .item("score", AttributeValue::N(input.score.to_string()))
            .item("level", AttributeValue::N(input.level.to_string()))
            .send()
            .await?;
    }

    // Scan table
    let result = client.scan().table_name(&table_name).send().await?;

    let leaderboard = result
        .items()
        .iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_s().unwrap();
            let score_str = item.get("score")?.as_n().unwrap();
            let level_str = item.get("level")?.as_n().unwrap();
            let score = score_str.parse::<u32>().ok()?;
            let level = level_str.parse::<usize>().ok()?;
            Some(ScoreEntry::add(name, score, level))
        })
        .collect::<Vec<_>>();

    let response = Output { leaderboard };

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::Text(serde_json::to_string(&response)?))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
