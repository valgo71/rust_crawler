use reqwest;
use quick_xml::Reader;
use quick_xml::events::Event;
use scraper::{Html, Selector};
use regex::Regex;
use std::collections::HashSet;
use std::io::Cursor;
use tokio;

#[tokio::main]
async fn main() {
    let base_url = "https://heygoody.com"; // 🔍 เปลี่ยน URL ที่ต้องการ
    println!(" Crawling Sitemap from: {}", base_url);

    //  โหลด URLs จาก Sitemap
    let sitemap_links = crawl_sitemap(base_url).await;

    println!(" Found {} URLs in Sitemap", sitemap_links.len());

    for url in &sitemap_links {
        println!(" Fetching: {}", url);
        match fetch_html(url).await {
            Some(html) => {
                let markdown = html_to_markdown(&html);
                println!("📄 Converted {} to Markdown ", url);
                println!("{}", markdown);
            }
            None => println!("❌ Failed to fetch: {}", url),
        }
    }
}

//  โหลด URLs จาก Sitemap XML (แก้ไข `read_text` เป็น `read_event`)
async fn crawl_sitemap(base_url: &str) -> HashSet<String> {
    let mut sitemap_links = HashSet::new();
    let sitemap_url = format!("{}/sitemap.xml", base_url);

    if let Ok(resp) = reqwest::get(&sitemap_url).await {
        if let Ok(body) = resp.text().await {
            let mut reader = Reader::from_reader(Cursor::new(body));
            let mut buf = Vec::new();

            while let Ok(event) = reader.read_event_into(&mut buf) {
                match event {
                    Event::Start(ref e) if e.name().as_ref() == b"loc" => {
                        if let Ok(Event::Text(text)) = reader.read_event_into(&mut Vec::new()) {
                            let link = String::from_utf8_lossy(&text.into_inner()).to_string();
                            sitemap_links.insert(link);
                        }
                    }
                    Event::Eof => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }

    sitemap_links
}

//  โหลด HTML ของหน้าเว็บ
async fn fetch_html(url: &str) -> Option<String> {
    if let Ok(resp) = reqwest::get(url).await {
        if let Ok(body) = resp.text().await {
            return Some(body);
        }
    }
    None
}

//  แปลง HTML เป็น Markdown
fn html_to_markdown(html: &str) -> String {
    let re_script = Regex::new(r"(?is)<script.*?</script>").unwrap();
    let re_style = Regex::new(r"(?is)<style.*?</style>").unwrap();
    let clean_html = re_script.replace_all(html, "").to_string();
    let clean_html = re_style.replace_all(&clean_html, "").to_string();

    let document = Html::parse_document(&clean_html);
    let selector = Selector::parse("body").unwrap();

    document
        .select(&selector)
        .next()
        .map(|elem| elem.text().collect::<Vec<_>>().join("\n"))
        .unwrap_or_else(|| "".to_string())
}
