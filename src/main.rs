mod card;

use std::collections::HashMap;
use crate::card::card::Card;
use indicatif::ProgressBar;
use regex;
use serde_json::Value;
use soup::prelude::*;
use std::{env, thread};
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args[1]);
    // let steam_inventory: String = request_page(&args[1])?;
    // format!(
    //     "https://steamcommunity.com/inventory/{}/753/6?count=5000",
    //     id
    // )
    // Load JSON from Steam API
    let steam_inventory = fs::read_to_string(Path::new("./src/results.json"))?;
    let json: Value = serde_json::de::from_str(&steam_inventory)?;

    let descriptions = json.get("descriptions").unwrap().as_array().unwrap();
    let mut dupes: Vec<String> = vec![];

    // Fetch duplicate cards
    {
        let assets = json.get("assets").unwrap().as_array().unwrap();
        let mut class_ids: Vec<String> = vec![];
        for x in assets {
            let class_id = x["classid"].to_string();
            if class_ids.contains(&class_id) {
                dupes.push(class_id);
                continue;
            }
            class_ids.push(class_id);
        }
    }
    let mut owned_cards: Vec<(String, String)> = vec![];
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
                println!("{}", dupe);
                sce_pages_to_fetch.push(description.get("market_fee_app").unwrap().to_string());
                owned_cards.push((
                    description.get("market_fee_app").unwrap().to_string(),
                    description.get("name").unwrap().to_string(),
                ));
            }
        }
    }
    // let sce_pages_to_fetch = vec!["507520"];
    let bar = ProgressBar::new(sce_pages_to_fetch.len() as u64);
    let mut parsed_cards: HashMap<String, Card> = HashMap::new();
    let chunks = sce_pages_to_fetch.chunks(5);
    let mut handles = vec![];
    for chunk in chunks {
        for sce_page in chunk {
            let handle = thread::spawn(move || {

            });
            handles.push(handle);
        }

    }

    // Fetch card info from steamcardexchange.net
    for page in sce_pages_to_fetch {

        bar.inc(1);
    }
    bar.finish();
    Ok(())
}

fn fetch_sce_cards() {

    let page_text = request_page(&format!(
        "https://steamcardexchange.net/index.php?inventorygame-appid-{}",
        &page
    ))?;

    println!("Requesting");
    let document = Soup::new(&page_text);
    println!("Requested");
    let game_name = document.tag("span").attr("class", "game-title").find().unwrap().text();

    for x in document
        .tag("div")
        .attr("class", "inventory-game-card-item")
        .find_all()
    {
        let name = x.tag("span").attr("class", "card-name").find();
        if name.is_none() {
            continue;
        }
        let name = name.unwrap().text();
        let stock = x.tag("span").attr("class", "card-amount").find().unwrap().text().replace("Stock: ", "");
        let mut price_worth = x.tag("span").attr("class", "card-price").find_all();
        let worth = price_worth.next().unwrap().text().replace("Worth: ", "");
        let price = price_worth.next().unwrap().text().replace("Price: ", "");
        let card: Card = Card {
            game_name: (&game_name).to_string(),
            game_id: page.to_string(),
            name: name.to_string(),
            stock,
            worth,
            price
        };
        println!("{:?}", card);
        // parsed_cards.insert(format!("{}-{}", page.to_string(), name), card);
    }
}

#[allow(dead_code)]
fn request_page(url: &str) -> anyhow::Result<String> {
    let client = reqwest::blocking::Client::builder().build()?;
    let results = client.get(url).send()?.text()?;
    Ok(results)
}
