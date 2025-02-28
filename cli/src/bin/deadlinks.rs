use etch_html::{file, visitor::{self}};
use etch_core::walk::FileWalker;
use log::{info, warn, error};
use env_logger;
use dotenv::dotenv;
use tokio;
use std::collections::{HashMap, HashSet};
use reqwest;
use url;
use chrono;
use std::io::{self, Write};

const ROOT_DIR: &str = "/Users/hectorcrean/typescript/OTS110_WebApp/src/content";

#[derive(Hash, Eq, PartialEq)]
struct LinkInfo {
    url: String,
    source_file: String,
    html_context: String,  // Add HTML context
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenv().ok();
    env_logger::init();
    info!("Starting dead link check in directory: {}", ROOT_DIR);

    let walker = FileWalker::new(["html"]);

    let mut links = HashSet::new();

    // First pass: collect all links with their sources
    let _ = walker.visit(ROOT_DIR, |path, _| {
        let relative_path = path.strip_prefix(ROOT_DIR)
            .unwrap_or(path)
            .display()
            .to_string();
        info!("Processing file: {}", relative_path);
        
        let visitor = visitor::LinkVisitor::new();
        let (_, visitor) = file::process_html_file(path, visitor)?;
        
        // Store each link with its source file and HTML context
        for (uuid, link) in visitor.links() {  // Assuming this method is added to LinkVisitor
            links.insert(LinkInfo {
                url: link.url.clone(),
                source_file: relative_path.clone(),
                html_context: link.text.clone(),
            });
        }
       
        Ok(())
    });

    // Track statistics
    let mut stats = HashMap::new();
    stats.insert("valid", 0);
    stats.insert("invalid", 0);
    stats.insert("unchecked", 0);

    // Create log file with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let log_path = format!("deadlinks_{}.md", timestamp);  // Change extension to .md
    let mut log_file = std::fs::File::create(&log_path)?;

    // Write Markdown header
    writeln!(log_file, "# Dead Links Check Report\n")?;
    writeln!(log_file, "**Date:** {}\n", timestamp)?;
    writeln!(log_file, "**Root Directory:** {}\n", ROOT_DIR)?;

    // Second pass: check all collected links
    let client = reqwest::Client::new();
    for link_info in links.iter() {
        let check_result = check_link(&client, &link_info.url).await;
        match check_result {
            LinkStatus::Valid => {
                // info!("✅ Valid link: {} (in {})", link_info.url, link_info.source_file);
                // writeln!(log_file, "## ✅ Valid Link\n")?;
                // writeln!(log_file, "- **URL:** {}", link_info.url)?;
                // writeln!(log_file, "- **File:** {}", link_info.source_file)?;
                // writeln!(log_file, "- **Context:**\n```html\n{}\n```\n", link_info.html_context)?;
                *stats.get_mut("valid").unwrap() += 1;
            }
            LinkStatus::Invalid(e) => {
                warn!("❌ Invalid link: {} (in {})", link_info.url, link_info.source_file);
                writeln!(log_file, "## ❌ Invalid Link\n")?;
                writeln!(log_file, "- **URL:** {}", link_info.url)?;
                writeln!(log_file, "- **File:** {}", link_info.source_file)?;
                writeln!(log_file, "- **Error:** {}", e)?;
                writeln!(log_file, "- **Context:**\n```html\n{}\n```\n", link_info.html_context)?;
                *stats.get_mut("invalid").unwrap() += 1;
            }
            LinkStatus::Unchecked => {
                info!("⚠️ Unchecked link: {} (in {})", link_info.url, link_info.source_file);
                writeln!(log_file, "## ⚠️ Unchecked Link\n")?;
                writeln!(log_file, "- **URL:** {}", link_info.url)?;
                writeln!(log_file, "- **File:** {}", link_info.source_file)?;
                writeln!(log_file, "- **Context:**\n```html\n{}\n```\n", link_info.html_context)?;
                *stats.get_mut("unchecked").unwrap() += 1;
            }
        }
    }

    // Write summary statistics in Markdown
    writeln!(log_file, "# Summary\n")?;
    writeln!(log_file, "- **Valid links:** {}", stats["valid"])?;
    writeln!(log_file, "- **Invalid links:** {}", stats["invalid"])?;
    writeln!(log_file, "- **Unchecked links:** {}", stats["unchecked"])?;

    info!("Dead link check completed successfully. Results written to {}", log_path);
    Ok(())
}

pub enum LinkStatus {
    Valid,
    Invalid(String),
    Unchecked,
}
// Move link checking logic to a separate async function
async fn check_link(client: &reqwest::Client, url: &str) -> LinkStatus {
    // Skip checking anchor links and empty URLs
    if url.is_empty() || url.starts_with('#') {
        return LinkStatus::Valid;
    }

    // Try to parse the URL
    let parsed_url = match url::Url::parse(url) {
        Ok(url) => url,
        Err(_) => {
            // Try adding https:// prefix if missing
            match url::Url::parse(&format!("https://{}", url)) {
                Ok(url) => url,
                Err(e) => return LinkStatus::Invalid(format!("Invalid URL: {}", e)),
            }
        }
    };

    // Only check http(s) URLs
    if !["http", "https"].contains(&parsed_url.scheme()) {
        return LinkStatus::Unchecked;
    }

    // First try HEAD request (faster)
    match client
        .head(parsed_url.as_str())
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            // Some servers don't support HEAD requests, fall back to GET
            if status.is_client_error() {
                return check_with_get(client, &parsed_url).await;
            }
            if status.is_success() || status.is_redirection() {
                LinkStatus::Valid
            } else {
                LinkStatus::Invalid(format!("HTTP status: {}", status))
            }
        }
        Err(_) => {
            // HEAD failed, try GET as fallback
            check_with_get(client, &parsed_url).await
        }
    }
}

async fn check_with_get(client: &reqwest::Client, url: &url::Url) -> LinkStatus {
    match client
        .get(url.as_str())
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.is_redirection() {
                LinkStatus::Valid
            } else {
                LinkStatus::Invalid(format!("HTTP status: {}", status))
            }
        }
        Err(e) => LinkStatus::Invalid(format!("Request failed: {}", e)),
    }
}

