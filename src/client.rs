use std::{fmt::Debug, sync::Arc};

use base64::Engine;
use futures::Future;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::{
    header::{self, HeaderMap},
    ClientBuilder, Method, RequestBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::{debug, trace};
use url::Url;

use crate::writer::Writer;

const DETECTION_START: u32 = 500;
const CHUNK: u32 = 100;

#[derive(Debug, Clone)]
pub struct Client {
    endpoint: Url,
    client: reqwest::Client,
}

impl Client {
    pub fn new(host: String, user: String, password: String) -> Self {
        let endpoint =
            Url::parse(format!("https://{host}/api/v1/").as_str()).unwrap_or_else(|_| {
                panic!(
                    "Unable to construct valid endpoint url with the provided host name \"{host}\""
                )
            });

        let token = base64::engine::general_purpose::URL_SAFE.encode(format!("{user}:{password}"));

        let mut auth_value = header::HeaderValue::from_str(format!("Basic {token}").as_str())
            .expect("Unable to construct auth header");

        auth_value.set_sensitive(true);

        let mut headers = HeaderMap::new();

        headers.insert(header::AUTHORIZATION, auth_value);

        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();

        Client { endpoint, client }
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.request(Method::GET, path)
    }

    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = self.endpoint.join(path).unwrap();

        self.client
            .request(method, url)
            .query(&[("format", "json")])
    }

    pub async fn get_resource_page(&self, resource: &str, take: u32, skip: u32) -> ResourcePage {
        self.get(resource)
            .query(&[("take", take), ("skip", skip)])
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    #[tracing::instrument]
    pub async fn detect_last_item(&self, resource: &str) -> u32 {
        // Find first empty page
        let mut right = DETECTION_START;
        let mut left = 0;

        loop {
            let response = self.get_resource_page(resource, 1, right).await;

            if response.items.is_empty() {
                break;
            }

            if response.next.is_none() {
                right += response.items.len() as u32;
                break;
            }

            left = right;
            right *= 2;
        }

        loop {
            let v = left + (right - left) / 2;

            let response = self.get_resource_page(resource, 1, v).await;

            if v == 0 || !response.items.is_empty() && response.next.is_none() {
                break v;
            }

            if response.items.is_empty() {
                right = v;
            } else {
                left = v;
            }
        }
    }

    #[tracing::instrument]
    pub async fn backup_resource<O>(&self, resource: &str, out: &mut O, pb: &Option<ProgressBar>)
    where
        O: Writer,
    {
        debug!(resource = resource, "starting resource backup");

        let mut skip = 0;

        if let Some(ref pb) = pb {
            let last = self.detect_last_item(resource).await;

            if last == 0 {
                return;
            }

            pb.set_prefix(String::from(resource));

            pb.set_length(last as u64);
        }

        out.write(format!("{{\"type\": \"{resource}\", \"items\": [").as_bytes())
            .await;

        loop {
            let page = self.get_resource_page(resource, CHUNK, skip).await;

            if page.items.is_empty() {
                break;
            }

            let s = serde_json::to_string(&page.items).unwrap();

            out.write(s[1..s.len() - 1].as_bytes()).await;

            if let Some(ref pb) = pb {
                pb.inc(page.items.len() as u64);
            }

            if page.next.is_none() {
                break;
            }

            skip += CHUNK;
        }

        out.write(b"]}").await;

        trace!(resource = resource, "resource backup complete");
    }

    pub async fn backup_all<C, Fut, T>(
        &self,
        streams: usize,
        progress: bool,
        resources: Vec<String>,
        out: C,
    ) where
        C: (Fn(String) -> Fut) + Send + Sync + 'static,
        Fut: Future<Output = T> + Send,
        T: Writer + Send,
    {
        let out = Arc::new(out);

        let rn = resources.len();

        let resources = Arc::new(Mutex::new(resources));

        let pb_style = ProgressStyle::with_template(
            "{spinner:.green} [{prefix}] [{wide_bar:.cyan/blue}] {pos}/{len}",
        )
        .unwrap();

        let (mpb, tpb) = if progress {
            let mpb = MultiProgress::new();

            let tpb = mpb.add(ProgressBar::new(rn as u64));

            tpb.set_style(pb_style.clone());

            (Some(mpb), Some(tpb))
        } else {
            (None, None)
        };

        let tpb = Arc::new(tpb);

        let mut js = JoinSet::new();

        for _ in 0..streams {
            let pb = if let Some(ref mpb) = mpb {
                let pb = ProgressBar::new(1);

                pb.set_style(pb_style.clone());

                Some(mpb.add(pb))
            } else {
                None
            };

            let resources = resources.clone();

            let client = self.clone();

            let tpb = tpb.clone();

            let out = out.clone();

            js.spawn(async move {
                while let Some(resource) = {
                    let mut m = resources.lock().await;

                    m.pop()
                } {
                    let mut writer = out(resource.clone()).await;

                    client.backup_resource(&resource, &mut writer, &pb).await;

                    if let Some(ref pb) = pb {
                        pb.reset();
                    }

                    if let Some(tpb) = &*tpb {
                        tpb.inc(1);
                    }
                }

                if let Some(ref pb) = pb {
                    pb.finish_and_clear();
                }
            });
        }

        while js.join_next().await.is_some() {}

        if let Some(tpb) = &*tpb {
            tpb.finish_and_clear();
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResourcePage {
    pub next: Option<String>,

    pub prev: Option<String>,

    pub items: Vec<Value>,
}
