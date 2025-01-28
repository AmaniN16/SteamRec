use reqwest::blocking::get;
use scraper::{Html, Selector};
use quick_xml::de::from_str;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use csv::Writer;

#[derive(Debug, Deserialize)]
struct Game {
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "appID")]
    app_id: String,
    #[serde(rename = "hoursOnRecord")]
    hours_on_record: String, 
}

#[derive(Debug, Deserialize)]
struct Profile {
    #[serde(rename = "steamID")]
    steam_id: String,
    #[serde(rename = "games")]
    games: Vec<Game>,
}

fn scrape_steam_profile(steam_id: &str) -> Result<Vec<Game>, Box<dyn Error>> {
    let url = format!("https://steamcommunity.com/id/{}/games?xml=1", steam_id);
    let response = get(&url)?;

    if response.status().is_success() {
        let text = response.text()?;
        println!("XML Response: {}", text); // Debugging: Print the XML response

        let profile: Profile = from_str(&text)?;

        let mut games = profile.games;
        games.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(games)
    } else {
        Err(format!("Failed to retrieve data. HTTP Status code: {}", response.status()).into())
    }
}

fn generate_csv(games: Vec<Game>) -> Result<(), Box<dyn Error>> {
    let url = "https://store.steampowered.com/search/?filter=topsellers";
    let response = get(url)?;
    let page_content = response.text()?;
    let doc = Html::parse_document(&page_content);

    let file = File::create("games_mine.csv")?;
    let mut writer = Writer::from_writer(file);
    writer.write_record(&["Name", "Published_Date", "Price", "Reviews", "Hours_on_Record", "App_Id", "Genre", "Developer"])?;

    let total_pages = doc
        .select(&Selector::parse("div.search_pagination_right a").unwrap())
        .last()
        .and_then(|elem| elem.text().next())
        .and_then(|text| text.parse::<usize>().ok())
        .unwrap_or(1);

    println!("Scraping data...");

    let mut count = 0;

    for page in 1..=total_pages {
        let page_url = format!("{}&page={}", url, page);
        let response = get(&page_url)?;
        let doc = Html::parse_document(&response.text()?);

        let game_selector = Selector::parse("div.responsive_search_name_combined").unwrap();
        let name_selector = Selector::parse("span.title").unwrap();
        let date_selector = Selector::parse("div.col.search_released.responsive_secondrow").unwrap();
        let price_selector = Selector::parse("div.discount_final_price").unwrap();
        let review_selector = Selector::parse("span.search_review_summary").unwrap();

        for game in doc.select(&game_selector) {
            let name = game.select(&name_selector).next().map(|e| e.text().collect::<String>()).unwrap_or_default();
            let published_date = game.select(&date_selector).next().map(|e| e.text().collect::<String>()).unwrap_or_default();
            let discount_price = game.select(&price_selector).next().map(|e| e.text().collect::<String>()).unwrap_or_default();
            let reviews_html = game.select(&review_selector).next().map(|e| e.value().attr("data-tooltip-html").unwrap_or("")).unwrap_or_default();
            let app_id = game
                                        .parent()
                                        .and_then(|e| e.value().as_element())
                                        .and_then(|elem| elem.attr("data-ds-appid"))
                                        .unwrap_or_default();

            let mut hours_on_record = "0".to_string();
            for profile_game in &games {
                if profile_game.name == name {
                    hours_on_record = profile_game.hours_on_record.clone();
                    count += 1;
                    break;
                }
            }

            let appdetails_url = format!("http://store.steampowered.com/app/{}", app_id);
            let appdetails_response = get(&appdetails_url)?;
            let app_doc = Html::parse_document(&appdetails_response.text()?);

            let genres_selector = Selector::parse("div#genresAndManufacturer span a").unwrap();
            let dev_selector = Selector::parse("div.dev_row a").unwrap();

            let genres = app_doc
                .select(&genres_selector)
                .map(|e| e.text().collect::<String>())
                .collect::<Vec<_>>()
                .join(",");

            let developer = app_doc
                .select(&dev_selector)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default();

            let reviews_percentage = reviews_html
                .split('%')
                .next()
                .and_then(|s| s.parse::<f32>().ok())
                .map(|p| p / 100.0)
                .unwrap_or(0.0);

            writer.write_record(&[
                name,
                published_date.trim().to_string(),
                discount_price.trim().to_string(),
                reviews_percentage.to_string(),
                hours_on_record,
                app_id.to_string(),
                genres,
                developer,
            ])?;

            if count == games.len() {
                return Ok(());
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let steam_id = "Amoneyyy";
    let games = scrape_steam_profile(steam_id)?;
    generate_csv(games)?;

    Ok(())
}