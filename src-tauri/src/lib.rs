use notify_rust::Notification;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::{process, thread};
use tokio::sync::Mutex;
use url::Url;

#[derive(Serialize, Deserialize, Clone)]
struct LinkNode {
    url: String,
    sub_urls: Vec<LinkNode>,
}

#[derive(Debug)]
struct CrawlError {
    message: String,
}

impl std::fmt::Display for CrawlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CrawlError {}

async fn normalize_url(base_url: &str, href: &str) -> Result<String, CrawlError> {
    if href.starts_with("http") {
        Ok(href.to_string())
    } else if href.starts_with("//") {
        Ok(format!("https:{}", href))
    } else if href.starts_with("/") {
        let base = Url::parse(base_url).map_err(|e| CrawlError {
            message: e.to_string(),
        })?;
        Ok(format!(
            "{}://{}{}",
            base.scheme(),
            base.host_str().unwrap_or(""),
            href
        ))
    } else {
        Url::parse(base_url).map_err(|e| CrawlError {
            message: e.to_string(),
        })?;
        Ok(format!("{}/{}", base_url.trim_end_matches('/'), href))
    }
}

async fn get_base_domain(url: &str) -> Result<String, CrawlError> {
    let parsed = Url::parse(url).map_err(|e| CrawlError {
        message: e.to_string(),
    })?;
    Ok(parsed.host_str().unwrap_or("").to_string())
}

async fn get_base_path(url: &str) -> Result<String, CrawlError> {
    let parsed = Url::parse(url).map_err(|e| CrawlError {
        message: e.to_string(),
    })?;

    let path = parsed.path();
    if path.is_empty() || path == "/" {
        return Ok("/".to_string());
    }

    // Remove trailing slash if exists
    let path = path.trim_end_matches('/');

    // Get the last segment of the path
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Ok("/".to_string());
    }

    Ok(format!("/{}", segments.last().unwrap()))
}

async fn fetch_links_recursively(
    url: &str,
    depth: u32,
    visited: Arc<Mutex<HashSet<String>>>,
    base_domain: &str,
    base_path: &str,
) -> Result<LinkNode, CrawlError> {
    // Check depth limit first
    if depth > 3 {
        return Ok(LinkNode {
            url: url.to_string(),
            sub_urls: vec![],
        });
    }

    let mut visited_guard = visited.lock().await;
    if visited_guard.contains(url) {
        return Ok(LinkNode {
            url: url.to_string(),
            sub_urls: vec![],
        });
    }
    visited_guard.insert(url.to_string());
    drop(visited_guard);

    let response = reqwest::get(url).await.map_err(|e| CrawlError {
        message: e.to_string(),
    })?;

    let html = response.text().await.map_err(|e| CrawlError {
        message: e.to_string(),
    })?;

    let html_clone = html.clone();
    let selector = Selector::parse("a").map_err(|e| CrawlError {
        message: e.to_string(),
    })?;

    let links = tokio::task::spawn_blocking(move || {
        let fragment = Html::parse_fragment(&html_clone);
        fragment
            .select(&selector)
            .filter_map(|link| link.value().attr("href").map(|h| h.to_string()))
            .collect::<Vec<String>>()
    })
    .await
    .map_err(|e| CrawlError {
        message: e.to_string(),
    })?;

    let mut sub_urls = Vec::new();

    for href in links {
        match normalize_url(url, &href).await {
            Ok(full_url) => {
                // Verifica se a URL está no mesmo domínio e contém o caminho base
                if full_url.contains(base_domain) && full_url != url && full_url.contains(base_path)
                {
                    let visited_guard = visited.lock().await;
                    if !visited_guard.contains(&full_url) {
                        drop(visited_guard);
                        // Only process if we haven't reached max depth
                        if depth < 3 {
                            match Box::pin(fetch_links_recursively(
                                &full_url,
                                depth + 1,
                                visited.clone(),
                                base_domain,
                                base_path,
                            ))
                            .await
                            {
                                Ok(node) => sub_urls.push(node),
                                Err(_) => continue,
                            }
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }

    println!("url: {}", url.to_string());

    sub_urls.iter().for_each(|sub_url| {
        println!("sub_url: {}", sub_url.url.to_string());
    });

    Ok(LinkNode {
        url: url.to_string(),
        sub_urls,
    })
}

#[tauri::command]
async fn open(app: tauri::AppHandle, url: &str) -> Result<LinkNode, String> {
    let visited = Arc::new(Mutex::new(HashSet::new()));

    let base_domain = get_base_domain(url).await.map_err(|e| e.to_string())?;
    let base_path = get_base_path(url).await.map_err(|e| e.to_string())?;

    let result = fetch_links_recursively(url, 0, visited, &base_domain, &base_path)
        .await
        .map_err(|e| e.to_string())?;

    let _window = tauri::WebviewWindowBuilder::new(
        &app,
        "result-graph",
        tauri::WebviewUrl::App("/#result".into()),
    )
    .center()
    .devtools(true)
    .inner_size(1200.0, 800.0)
    .min_inner_size(600.0, 400.0)
    .max_inner_size(1600.0, 1200.0)
    .title("Graphix")
    .build()
    .unwrap();

    thread::spawn(|| {
        Notification::new()
            .summary("Graphix")
            .body("Grafo gerado com sucesso")
            .icon("✅")
            .show()
            .unwrap()
            .on_close(|_reason| {});
    });

    Ok(result)
}

async fn get_page_html_for(graph: &mut Graph) -> Result<String, reqwest::Error> {
    let response = reqwest::get(&graph.url).await?;
    let body: String;

    if !response.status().is_success() {
        //
    }

    body = response.text().await?;

    println!("{}", body);

    Ok(body)
}

#[tauri::command]
async fn notification() {
    println!("Tentando exibir notificação");

    // Usando notify-rust em nova thread
    thread::spawn(|| {
        match Notification::new()
            .summary("Time is running out")
            .body("This will go away.")
            .icon("✅")
            .show()
        {
            Ok(handle) => {
                println!("Notificação via notify-rust exibida com sucesso");
                handle.on_close(|| println!("Notificação fechada"));
            }
            Err(e) => println!("Erro ao exibir notificação via notify-rust: {}", e),
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![open, notification])
        .setup(|_app| {
            println!("Setup iniciado");

            thread::spawn(|| {
                Notification::new()
                    .summary("Teste")
                    .body("This will go away.")
                    .icon("../icons/32x32.png")
                    .show()
                    .unwrap()
                    .on_close(|| println!("Notificação setup fechada"));
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// enum Type {
//     Teste,
//     Teste2,
// }

struct Graph {
    url: String,
    // force: bool,
}

impl Graph {
    fn new(url: String) -> Self {
        Graph { url }
    }

    fn get_subdomain(&self) -> String {
        let parts = &self.url.split('.').collect::<Vec<&str>>();
        parts[0].to_string()
    }
}
