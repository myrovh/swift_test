use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::path::Path;

pub type PhoneNumber = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Address {
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub postcode: String,
    pub country: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub first_name: String,
    pub last_name: String,
    pub phone_number: PhoneNumber,
    pub address: Option<Address>,
}

impl Eq for Contact {}

impl PartialEq for Contact {
    fn eq(&self, other: &Self) -> bool {
        self.phone_number == other.phone_number
    }
}

impl Hash for Contact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.phone_number.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhoneBook {
    contacts: HashSet<Contact>,
}

impl PhoneBook {
    pub fn new() -> PhoneBook {
        PhoneBook {
            contacts: HashSet::new(),
        }
    }

    pub fn new_from_file(path: &OsStr) -> Result<PhoneBook> {
        let path = Path::new(path);

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let u = serde_json::from_reader(reader)?;

        Ok(u)
    }

    pub fn save_to_file(&self, path: &OsStr) -> Result<()> {
        let path = Path::new(path);

        let file = File::create(&path)?;

        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    pub fn insert_contact(&mut self, contact: Contact) -> Result<()> {
        is_valid_phone_number(contact.phone_number.as_str())?;

        if self.contacts.insert(contact.clone()) != true {
            return Err(anyhow!("number already exists unable insert"));
        }

        Ok(())
    }

    pub fn replace_contact(&mut self, contact: Contact) -> Result<()> {
        is_valid_phone_number(contact.phone_number.as_str())?;

        return match self.contacts.replace(contact) {
            Some(_) => Ok(()),
            None => Err(anyhow!("unable to update contact")),
        };
    }

    pub fn delete_contact(&mut self, number: String) -> Result<()> {
        is_valid_phone_number(number.as_str())?;

        if self.contacts.remove(&Contact {
            first_name: "".to_string(),
            last_name: "".to_string(),
            phone_number: number,
            address: None,
        }) {
            return Ok(());
        };

        Err(anyhow!("unable to delete contact"))
    }

    pub fn find_phone_number(&self, number: String) -> Result<&Contact> {
        is_valid_phone_number(number.as_str())?;

        self.contacts
            .iter()
            .find(|contact| contact.phone_number == number)
            .ok_or(anyhow!("no contact found"))
    }

    pub fn find_name(&self, first: Option<String>, last: Option<String>) -> Vec<&Contact> {
        self.contacts
            .iter()
            .filter(|contact| {
                contact.first_name == first.clone().unwrap_or_default()
                    || contact.last_name == last.clone().unwrap_or_default()
            })
            .collect::<Vec<_>>()
    }

    pub fn find_city(&self, city: String) -> Vec<&Contact> {
        self.contacts
            .iter()
            //TODO try and not clone this?
            .filter(|contact| match contact.address.clone() {
                Some(address) => address.city == city,
                _ => false,
            })
            .collect::<Vec<_>>()
    }
}

fn is_valid_phone_number(number: &str) -> Result<()> {
    if number.len() == 10 {
        return Ok(());
    }

    Err(anyhow!("phone number must be 10 characters long"))
}
