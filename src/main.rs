#[macro_use]
extern crate lazy_static;

use std::{env, thread};
use std::sync::{Arc, Mutex};

use indicatif::ProgressBar;
use scraper::{Html, Selector};
use serde_json::Value;

use crate::card::card::Card;

mod card;

lazy_static! {
    static ref CHUNK_SIZE: usize = 5;

    static ref GAME_NAME_SELECTOR: Selector = Selector::parse("h2.empty").unwrap();
    static ref CARD_INFO_SELECTOR: Selector = Selector::parse("div.inventory-game-card-item").unwrap();
    static ref CARD_NAME_SELECTOR: Selector = Selector::parse("span.card-name").unwrap();
    static ref CARD_STOCK_SELECTOR: Selector = Selector::parse("span.card-amount").unwrap();
    static ref CARD_WORTH_PRICE_SELECTOR: Selector = Selector::parse("span.card-price").unwrap();
}


fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let steam_inventory: String = request_page(&format!(
        "https://steamcommunity.com/inventory/{}/753/6?count=5000",
        args[1]
    ))?;

    // Load JSON from Steam API
    // let steam_inventory = fs::read_to_string(Path::new("./src/results.json"))?;
    let json: Value = serde_json::de::from_str(&steam_inventory)?;

    let descriptions = json.get("descriptions").unwrap().as_array().unwrap();
    let mut dupes: Vec<String> = vec![];

    // Fetch duplicate cards
    {
        let assets = json.get("assets").unwrap().as_array().unwrap();
        let mut class_ids: Vec<String> = vec![];
        for asset in assets {
            let class_id = asset["classid"].to_string();
            if class_ids.contains(&class_id) {
                dupes.push(class_id);
                continue;
            }
            class_ids.push(class_id);
        }
    }
    let owned_cards = Arc::new(Mutex::new(vec![]));
    let mut sce_pages_to_fetch: Vec<String> = vec![];

    // Find description of duplicates.
    for dupe in dupes {
        for description in descriptions {
            if description.get("classid").unwrap().to_string() == dupe
                && description.get("marketable").unwrap().to_string() == "1"
                && description
                    .get("tags")
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .get("localized_tag_name")
                    .unwrap()
                    != "Emoticon"
            {
                sce_pages_to_fetch.push(description.get("market_fee_app").unwrap().to_string());
                owned_cards.lock().unwrap().push((
                    description.get("market_fee_app").unwrap().to_string(),
                    description.get("name").unwrap().to_string().replace("\"",""),
                ));
            }
        }
    }
    let bar = Arc::new(Mutex::new(ProgressBar::new(sce_pages_to_fetch.len() as u64)));
    let chunks = sce_pages_to_fetch.chunks(*CHUNK_SIZE);
    let mut results = vec![];
    for chunk in chunks {
        let mut handles = vec![];

        for sce_page in chunk {
            let a = format!("{}", sce_page);
            let pb = Arc::clone(&bar);
            let duped = Arc::clone(&owned_cards);
            let handle = thread::spawn(move || {
                let result = fetch_duplicates(a, duped).unwrap();
                pb.lock().unwrap().inc(1);
                result
            });
            handles.push(handle);
        }
        for handle in handles {
            let mut map = handle.join().unwrap();
            results.append(&mut map);
        }
    }
    bar.lock().unwrap().finish();
    for card in results {
        println!("{:?}", card);
    }

    Ok(())
}

fn fetch_duplicates(page: String, dupes_arc: Arc<Mutex<Vec<(String, String)>>>) -> anyhow::Result<Vec<Card>> {
    let page_text = request_page(&format!(
        "https://steamcardexchange.net/index.php?inventorygame-appid-{}",
        &page
    ))?;
    let lock = dupes_arc.lock().unwrap();
    let dupes = lock.clone();
    drop(lock);
    let mut results: Vec<Card> = vec![];
    let document = Html::parse_document(&page_text);

    let game_name = document.select(&GAME_NAME_SELECTOR).nth(0).unwrap().inner_html();
    for element in document.select(&CARD_INFO_SELECTOR) {
        let name = match element.select(&CARD_NAME_SELECTOR).nth(0) {
            Some(name) => name.inner_html(),
            None => continue
        };
        if !dupes.contains(&(page.to_string(), name.to_string())) {
            continue;
        }
        let mut worth_price = element.select(&CARD_WORTH_PRICE_SELECTOR);
        let worth = worth_price.nth(0).unwrap().inner_html().replace("Worth: ", "");
        let price = worth_price.nth(0).unwrap().inner_html().replace("Price: ", "");

        if worth.contains("Overstocked") {
            continue;
        }
        let stock = element.select(&CARD_STOCK_SELECTOR).nth(0).unwrap().inner_html().replace("Stock: ", "");


        let card = Card {
            game_id: page.to_string(),
            game_name: game_name.to_string(),
            name,
            price,
            worth,
            stock,
        };
        results.push(card);
    }

    Ok(results)
}

fn request_page(url: &str) -> anyhow::Result<String> {
    let client = reqwest::blocking::Client::builder().build()?;
    let results = client.get(url).send()?.text()?;
    Ok(results)
}
