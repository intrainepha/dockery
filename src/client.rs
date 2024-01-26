#![deny(clippy::all)]

use chrono::{DateTime, Utc};
use humantime::format_duration;
use prettytable::{row, Table};
use regex::Regex;
use reqwest;
use serde::Deserialize;
use std::env;
use std::result::Result;
use std::string::String;

#[derive(Debug, Deserialize)]
struct Catalog {
    #[serde(default)]
    repositories: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct TagInfo {
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Config {
    digest: String,
}

#[derive(Debug, Deserialize)]
struct Comp {
    created: String,
}

#[derive(Debug, Deserialize)]
struct History {
    #[serde(rename = "v1Compatibility")]
    comp: String,
}

#[derive(Debug, Deserialize)]
struct ManifestHistory {
    history: Vec<History>,
}

#[derive(Debug, Deserialize)]
struct Layer {
    size: u64,
}

#[derive(Debug, Deserialize)]
struct ManifestInfo {
    config: Config,
    layers: Vec<Layer>,
}

pub struct Client {
    uri: String,
    client: reqwest::Client,
}

impl Client {
    pub async fn new() -> Self {
        let key = "DOCKERY";
        let uri = match env::var(key) {
            Ok(v) => v,
            Err(_) => String::from("0.0.0.0:5000"),
        };
        let client = match Client::get(&uri).await {
            Ok(c) => c,
            Err(_) => panic!("Connection failed: {}", &uri),
        };
        Client {
            uri: uri,
            client: client,
        }
    }

    async fn get(uri: &str) -> Result<reqwest::Client, reqwest::Error> {
        let url = format!("http://{}/v2", uri);
        let client = reqwest::Client::new();
        match client.get(&url).send().await {
            Ok(res) => match res.status().as_u16() {
                200..=299 => {
                    println!("Registry: {}", uri);
                }
                status_code => {
                    eprintln!("Connection error: Unexpected status code {}", status_code);
                    return res.error_for_status_ref().map(|_| (client));
                }
            },
            Err(e) => {
                eprintln!("Connection error: {}", e);
                return Err(e);
            }
        }
        Ok(client)
    }

    pub async fn images(&self) -> reqwest::Result<()> {
        let catalog = self.get_catalog().await?;
        let mut table = Table::new();
        let re = Regex::new(r"(\d)([a-zA-Z])").unwrap();
        table.add_row(row!["REPOSITORY", "TAG", "IMAGE ID", "CREATED", "SIZE"]);
        for repository in catalog.repositories.unwrap_or_default() {
            let tag_info = self.get_tags(&repository).await?;
            for tag in tag_info.tags.unwrap_or_default() {
                let (image_id, created, image_size) =
                    self.get_image_info(&repository, &tag).await?;
                let human_time = to_human_time(&created).unwrap();
                let parts: Vec<&str> = human_time.split_whitespace().collect();
                let extracted_part = parts.get(0).cloned().unwrap_or_else(|| human_time.as_str());
                let created_from = format!("{} ago", extracted_part);
                table.add_row(row![
                    repository,
                    tag,
                    &image_id[..12],
                    re.replace_all(&created_from, "$1 $2"),
                    format!("{:.2}GB", image_size),
                ]);
            }
        }

        table.printstd();
        Ok(())
    }

    pub async fn rmi(&self, repo: &str, tag: &str) -> reqwest::Result<()> {
        let mut url = format!("http://{}/v2/{}/manifests/{}", self.uri, repo, tag);
        let res = self
            .client
            .get(&url)
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await?;
        // let digest = res.headers()["docker-content-digest"].to_str().unwrap();
        let digest = match res.headers().get("docker-content-digest") {
            Some(value) => value.to_str().unwrap(),
            None => panic!("Image not found: {}:{}", repo, tag),
        };
        url = format!("http://{}/v2/{}/manifests/{}", self.uri, repo, &digest);
        let res = self
            .client
            .delete(&url)
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await?;
        match res.status().as_u16() {
            200..=299 => {
                println!("{repo}:{tag}");
            }
            _ => {
                eprintln!(
                    "Failed to remove image {repo}:{tag} with status code={}",
                    res.status().as_str()
                );
            }
        }
        Ok(())
    }

    async fn get_catalog(&self) -> Result<Catalog, reqwest::Error> {
        let url = format!("http://{}/v2/_catalog", self.uri);
        let res = self.client.get(url).send().await?;
        let body = res.text().await?;
        let catalog: Catalog = serde_json::from_str(&body).unwrap();
        Ok(catalog)
    }

    async fn get_tags(&self, repo: &str) -> Result<TagInfo, reqwest::Error> {
        let url = format!("http://{}/v2/{}/tags/list", self.uri, repo);
        let res = self.client.get(&url).send().await?;
        let body = res.text().await?;
        let tag_info: TagInfo = serde_json::from_str(&body).unwrap();
        Ok(tag_info)
    }

    async fn get_image_info(
        &self,
        repo: &str,
        tag: &str,
    ) -> reqwest::Result<(String, String, f64)> {
        let url = format!("http://{}/v2/{}/manifests/{tag}", self.uri, repo);
        let mut res = self.client.get(&url).send().await?;
        let mut body = res.text().await?;
        let mani: ManifestHistory = serde_json::from_str(&body).unwrap();
        let comp: Comp = serde_json::from_str(&mani.history[0].comp).unwrap();
        let created_time = comp.created;
        res = self
            .client
            .get(&url)
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.list.v2+json",
            )
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await?;
        body = res.text().await?;
        let mani: ManifestInfo = serde_json::from_str(&body).unwrap();
        let size: u64 = mani.layers.iter().map(|layer| layer.size).sum();
        let size_in_gb = size as f64 / (1024.0 * 1024.0 * 1024.0);
        let digest: Vec<&str> = mani.config.digest.split(":").collect();
        let image_id = digest.last().unwrap();
        Ok(((*image_id).to_string(), created_time, size_in_gb))
    }
}

fn to_human_time(time_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parsed_time = DateTime::parse_from_rfc3339(time_str);
    match parsed_time {
        Ok(datetime) => {
            let duration = Utc::now().signed_duration_since(datetime);
            let std_duration = duration
                .to_std()
                .expect("Failed to convert to std::time::Duration");
            let human_readable_time = format_duration(std_duration);
            Ok(format!("{}", human_readable_time))
        }
        Err(err) => {
            eprintln!("Error parsing time: {}", err);
            Err(err.into())
        }
    }
}
