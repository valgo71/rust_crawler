use std::collections::HashSet;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Client;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    let sitemap_url = "https://heygoody.com/sitemap.xml";
    println!(" Crawling Sitemap from: {}", sitemap_url);

    let urls = fetch_sitemap_urls(sitemap_url).await;

    println!(" Found {} URLs in Sitemap", urls.len());
    for url in &urls {
        println!("📄 {}", url);
    }
}

async fn fetch_sitemap_urls(sitemap_url: &str) -> HashSet<String> {
    let mut urls = HashSet::new();
    let client = Client::builder()
        .timeout(Duration::from_secs(10)) //  ตั้ง timeout 10 วินาที
        .build()
        .unwrap();

    println!(" Sending request...");
    let response = timeout(Duration::from_secs(10), client.get(sitemap_url)
        .header("User-Agent", "Mozilla/5.0 (compatible; RustCrawler/1.0; +https://heygoody.com)")
        .send()
    ).await;

    match response {
        Ok(Ok(resp)) => {
            println!("Response received!");

            // ตรวจสอบ Content-Type
            let content_type = resp.headers().get("Content-Type").map(|v| v.to_str().unwrap_or("")).unwrap_or("");
            println!(" Content-Type: {}", content_type);
            if !content_type.contains("xml") {
                println!(" This is not an XML file! It may be an error page.");
                return urls;
            }

            if let Ok(xml) = resp.text().await {
                println!(" First 500 chars of Sitemap:\n{}", &xml.chars().take(500).collect::<String>());

                let mut reader = Reader::from_str(&xml);
                reader.trim_text(true);

                while let Ok(event) = reader.read_event() {  
                    match event {
                        Event::Start(ref e) if e.name().as_ref() == b"loc" => {
                            if let Ok(text) = reader.read_text(e.name()) {
                                println!(" Found URL: {}", text);
                                urls.insert(text.to_string());
                            }
                        }
                        Event::Eof => break, // จบ XML แล้วให้หยุด loop
                        _ => {}
                    }
                }
            }
        }
        _ => {
            println!(" Request Timeout: โหลด Sitemap ไม่สำเร็จ");
        }
    }

    urls
}
