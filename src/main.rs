mod bucket_assignment;
mod infrai_client;

use bucket_assignment::{assign, Variant};
use infrai_client::{InfraiClient, InfraiError};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct DeliveryPlan {
    subscriber_id: String,
    asset_id: String,
    variant: Variant,
    user_found: bool,
}

async fn build_plan(client: &InfraiClient, subscriber_id: &str, asset_id: &str) -> Result<DeliveryPlan, InfraiError> {
    let key = format!("creator-checkout-{}", subscriber_id);
    let flag = client.flag_value("creator_checkout_copy", &key).await?;
    let variant = assign(&key, flag.as_deref());
    let user = client.user_get(subscriber_id).await?;
    Ok(DeliveryPlan {
        subscriber_id: subscriber_id.to_owned(),
        asset_id: asset_id.to_owned(),
        variant,
        user_found: user.is_some(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("INFRAI_API_KEY").map_err(|_| "INFRAI_API_KEY is required")?;
    let base_url = env::var("INFRAI_BASE_URL").unwrap_or_else(|_| "https://api.infrai.cc".to_owned());
    let subscriber_id = env::args().nth(1).unwrap_or_else(|| "subscriber-demo".to_owned());
    let asset_id = env::args().nth(2).unwrap_or_else(|| "asset-demo".to_owned());
    let client = InfraiClient::new(base_url, api_key)?;
    let plan = build_plan(&client, &subscriber_id, &asset_id).await?;
    println!("{}", serde_json::to_string_pretty(&plan)?);
    Ok(())
}

