use prettytable::{row, Table};
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
        match self.get_catalog().await {
            Ok(catalog) => {
                let mut table = Table::new();
                table.add_row(row!["REPOSITORY", "TAG", "IMAGE ID", "CREATED", "SIZE"]);
                for c in catalog.repositories.unwrap_or_default() {
                    match self.get_tags(&c).await {
                        Ok(tag_info) => {
                            for t in tag_info.tags.unwrap_or_default() {
                                match self.get_image_info(&c, &t).await {
                                    Ok((image_id, created, image_size)) => {
                                        let created_sec: Vec<&str> = created.split(".").collect();
                                        table.add_row(row![
                                            c,
                                            t,
                                            &image_id[..12],
                                            created_sec[0],
                                            format!("{:.2}GB", image_size)
                                        ]);
                                    }
                                    Err(err) => eprintln!("Error: {}", err),
                                }
                            }
                        }
                        Err(err) => eprintln!("Error: {}", err),
                    }
                }
                table.printstd();
            }
            Err(err) => eprintln!("Error: {}", err),
        }
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
