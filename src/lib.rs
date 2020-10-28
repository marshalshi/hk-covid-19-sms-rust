use chrono::prelude::*;
use itertools::Itertools;
use reqwest;
use reqwest::header::CONTENT_TYPE;
use serde::{self, Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;
use twilio;

pub mod constants;
use crate::constants::*;

fn string_to_date<'de, D>(deserializer: D) -> Result<Date<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let d = NaiveDate::parse_from_str(&s, "%d/%m/%Y").unwrap();
    Ok(Date::from_utc(d, Utc))
}

// String to Int
fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}

// Gender and how to parse
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
enum Gender {
    Male,
    Female,
}

fn gender_from_str<'de, D>(deserializer: D) -> Result<Gender, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s == "M" {
        Ok(Gender::Male)
    } else {
        Ok(Gender::Female)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Entry {
    #[serde(rename(deserialize = "Case no."), deserialize_with = "from_str")]
    case_no: u32,
    #[serde(rename(deserialize = "Age"))]
    age: u32,
    #[serde(
        rename(deserialize = "Gender"),
        deserialize_with = "gender_from_str"
    )]
    gender: Gender,

    #[serde(
        rename(deserialize = "Report date"),
        deserialize_with = "string_to_date"
    )]
    date: Date<Utc>,
}

#[derive(Debug)]
pub struct Cases {
    pub url: String,
    pub entries: Vec<Entry>,
}

impl Cases {
    pub fn new(url: &str) -> Self {
        Cases {
            url: url.to_string(),
            entries: Vec::new(),
        }
    }

    pub async fn fetch(&mut self) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        let res = client
            .get(&self.url)
            .header(CONTENT_TYPE, "binary/octet-stream")
            .send()
            .await?
            .text()
            .await?;
        let entries: Vec<Entry> = serde_json::from_str(&res).unwrap();
        self.entries = entries;
        Ok(())
    }

    fn date_cases(&self, date: &Date<Utc>) -> Vec<Entry> {
        let mut c = Vec::new();
        for en in self.entries.iter().rev() {
            if en.date == *date {
                c.push((*en).clone());
            } else {
                break;
            }
        }
        c
    }

    pub fn show_date_cases(&self, date: &Date<Utc>) -> String {
        let mut msg = "".to_string();
        let date_cases = self.date_cases(&date);
        msg += &format!("HK Covid-19 cases ({})\n", date);

        for (key, group) in &date_cases
            .iter()
            .group_by(|&elt| elt.gender == Gender::Male)
        {
            msg += &format!(
                "{:?}:\t{:?}\n",
                if key { Gender::Male } else { Gender::Female },
                group.count()
            );
        }

        msg += &format!("Total:\t{:?}", date_cases.len());
        println!("{}", msg);
        msg
    }

    pub async fn send_msg(msg: &str) -> Result<(), twilio::TwilioError> {
        let client = twilio::Client::new(SID, TOKEN);
        client
            .send_message(twilio::OutboundMessage::new(FROM, TO, msg))
            .await?;
        Ok(())
    }
}
