use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenv().ok();
	let zone_id = std::env::var("ZONE_ID").expect("ZONE_ID is not set");
	let access_token = std::env::var("ACCESS_TOKEN").expect("ACCESS_TOKEND is not set");
	let a_records_to_update =
		std::env::var("A_RECORDS_TO_UPDATE").expect("A_RECORDS_TO_UPDATE is not set");
	let a_records_to_update = a_records_to_update.split(",").collect::<Vec<_>>();
	let dns_records = get_dns_records(&zone_id, &access_token).await?;
	let public_ip = get_public_ip().await?;

	for dns_record in dns_records {
		if dns_record._type == "A" && a_records_to_update.contains(&dns_record.name.as_str()) {
			update_dns_record(&zone_id, &dns_record, public_ip.clone(), &access_token).await?;
			println!("Setting {} to {}", dns_record.name, public_ip);
		}
	}

	Ok(())
}

async fn get_public_ip() -> anyhow::Result<String> {
	Ok(Client::new()
		.get("https://am.i.mullvad.net")
		.header("User-agent", "curl")
		.send()
		.await?
		.text()
		.await?
		.trim()
		.to_string())
}

async fn get_dns_records(zone_id: &str, access_token: &str) -> anyhow::Result<Vec<DnsRecord>> {
	let url = format!(
		"https://api.cloudflare.com/client/v4/zones/{}/dns_records",
		zone_id
	);
	let response = Client::new()
		.get(&url)
		.bearer_auth(access_token)
		.send()
		.await?
		.json::<CloudflareResponse<Vec<DnsRecord>>>()
		.await?;

	if response.success {
		Ok(response.result.unwrap())
	} else {
		println!("{:?}", response.errors);
		panic!("Failed to fetch dns records");
	}
}

#[derive(Debug, Serialize)]
struct UpdateDnsRecord {
	#[serde(rename = "type")]
	_type: String,
	name: String,
	content: String,
	ttl: i32,
	proxied: bool,
}

async fn update_dns_record(
	zone_id: &str,
	record: &DnsRecord,
	record_value: String,
	access_token: &str,
) -> anyhow::Result<()> {
	let url = format!(
		"https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
		zone_id, record.id
	);
	let response = Client::new()
		.put(&url)
		.bearer_auth(access_token)
		.json(&UpdateDnsRecord {
			_type: record._type.clone(),
			name: record.name.clone(),
			content: record_value,
			ttl: 3600,
			proxied: false,
		})
		.send()
		.await?
		.json::<CloudflareResponse<DnsRecord>>()
		.await?;

	if response.success {
		Ok(())
	} else {
		println!("{:?}", response.errors);
		panic!("Failed to fetch dns records");
	}
}

#[derive(Debug, Deserialize)]
struct DnsRecord {
	id: String,
	#[serde(rename = "type")]
	_type: String,
	name: String,
}

#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
	result: Option<T>,
	success: bool,
	errors: Vec<CloudflareError>,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
	code: i64,
	message: String,
}
