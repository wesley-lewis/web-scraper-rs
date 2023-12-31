use reqwest::StatusCode;
use chrono;
use std::fs::File;
use std::io::Write;
use regex::Regex;
use scraper::{Html, Selector};

mod utils;
mod models;

#[tokio::main]
async fn main() {
    let client = utils::get_client();
    let domain_name = "finance.yahoo.com";
    let url = format!("https://{}", domain_name);
    let result = client.get(url).send().await.unwrap();

    let raw_html = match result.status() {
        StatusCode::OK => result.text().await.unwrap(),
        _ => panic!("Something went wrong"),
    };

    save_raw_html(&raw_html, domain_name);

    let document = Html::parse_document(&raw_html);
    let article_selector = Selector::parse("a.js-content-viewer").unwrap();
    let h2select = Selector::parse("h2").unwrap();
    let h3select = Selector::parse("h3").unwrap();

    let mut article_list: Vec<models::ArticleData> = Vec::new();
    for element in document.select(&article_selector) {
        let inner = element.inner_html().to_string();
        let mut h2el = element.select(&h2select);
        let mut h3el = element.select(&h3select);
        let get_text_re = Regex::new(r"->.*<").unwrap();
        let find_replace_re = Regex::new(r"[-><]").unwrap();
        
        let href = match element.value().attr("href") {
            Some(target_url) => target_url,
            _ => "no url found",
        };

        match h2el.next() {
            Some(elref) => {
                let title = elref.inner_html().to_string();
                println!("H2: {}", &elref.inner_html().to_string());
                println!("Link: {}", &href);

                article_list.push(models::ArticleData {
                    article_title: title,
                    url_link: href.to_string(),
                    domain_name: domain_name.to_string(),
                });
                continue;
            },

            _ => {}
        }

        match h3el.next() {
            Some(elref) => {
                let title = elref.inner_html().to_string();
                println!("H3: {}", &elref.inner_html().to_string());
                println!("Link: {}", &href);

                article_list.push(models::ArticleData {
                    article_title: title, 
                    url_link: href.to_string(),
                    domain_name: domain_name.to_string(),
                });
                continue;
            }
            _ => {}
        }

        match get_text_re.captures_iter(&inner).next() {
            Some(cap) => {
                let replaced = find_replace_re.replace_all(&cap[0], "");
                println!("Regex: {}", &replaced);
                println!("Link: {}", &href);

                article_list.push(models::ArticleData {
                    article_title: replaced.to_string(), 
                    url_link: href.to_string(),
                    domain_name: domain_name.to_string(),
                });
            }
            _ => {
                println!("nothing found");
            }
        };

    //     println!("Title: {}", &inner);
    //     println!("Link: {}", &href);

    }
    save_article_list(&article_list, domain_name);
    println!("Number of articles scraped: {}", article_list.len());
}

fn save_raw_html(raw_html: &str, domain_name: &str) {
    let dt = chrono::Local::now();
    let filename = format!("{}_{}.html", domain_name, dt.format("%Y-%m-%d_%H.%M.%S"));
    let mut writer = File::create(&filename).unwrap();
    write!(&mut writer, "{}", &raw_html).unwrap(); 
}

fn save_article_list(article_list: &Vec<models::ArticleData>, domain_name: &str) {
    let dt = chrono::Local::now();
    let filename = format!("{}_{}.json", domain_name, dt.format("%Y-%m-%d_%H.%M.%S"));
    let mut writer = File::create(&filename).unwrap();
    write!(&mut writer, 
           "{}", 
           &serde_json::to_string(&article_list).unwrap()
           )
            .unwrap();

    
}
