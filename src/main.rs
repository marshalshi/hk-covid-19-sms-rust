use chrono::prelude::*;
use covid_sms::constants::API_URL;
use covid_sms::Cases;
use std::env;

#[tokio::main]
async fn main() {
    // 1. parse args and get date
    let args: Vec<String> = env::args().collect();
    let date = if args.len() >= 2 {
        let s = &args[1];
        let d = NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .expect("Format has to be %Y-%m-%d");
        Date::from_utc(d, Utc)
    } else {
        Utc::now().date()
    };

    // 2. fetch data
    let mut cases = Cases::new(API_URL);
    cases.fetch().await.unwrap();
    let msg = cases.show_date_cases(&date);
    Cases::send_msg(&msg).await.unwrap();
}
