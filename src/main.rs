use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use phone_book::*;
use std::ffi::OsStr;
use std::process::exit;

/// Simple phone book manager
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short = 'f', default_value_t = String::from("phone_book.json"), value_parser, value_hint = clap::ValueHint::DirPath)]
    /// File to save and load the json from
    file: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates json file (this file needs to exist before other commands can be run)
    Init {},
    /// Add a new contact into the json file
    Add {
        #[clap(value_parser)]
        /// first name
        first: String,
        #[clap(value_parser)]
        /// last name
        last: String,
        /// phone number (must be exactly ten digits)
        phone_number: String,
        #[clap(value_parser = parse_address)]
        address: Option<Address>,
    },
    /// Modify an existing contact
    Update {
        /// phone number (must be exactly ten digits)
        #[clap(value_parser)]
        phone_number: String,
        #[clap(short = 'f', value_parser)]
        /// first name
        first: Option<String>,
        #[clap(short = 'l', value_parser)]
        /// last name
        last: Option<String>,
        #[clap(short = 'a', value_parser = parse_address)]
        address: Option<Address>,
    },
    Delete {
        /// phone number (must be exactly ten digits)
        #[clap(value_parser)]
        phone_number: String,
    },
    Search(Search),
}

#[derive(Args)]
struct Search {
    #[clap(subcommand)]
    command: SearchCommands,
}

#[derive(Subcommand)]
enum SearchCommands {
    /// search for a contact using phone number
    Phone {
        /// phone number (must be exactly ten digits)
        phone_number: String,
    },
    /// search for a contact using first and last names
    Name {
        /// first name
        #[clap(short = 'f', value_parser)]
        first: Option<String>,
        /// last name
        #[clap(short = 'l', value_parser)]
        last: Option<String>,
    },
    City {
        #[clap(value_parser)]
        city: String,
    },
    Fuzzy {
        #[clap(value_parser)]
        search: String,
    },
    /// returns contacts for which the given prefix matches the beginning of a contact's phone number
    Prefix {
        #[clap(value_parser)]
        search: String,
    },
}

/// `[street address], [city], [state/province], [zip code], [country]`
fn parse_address(s: &str) -> Result<Address> {
    let split: Vec<&str> = s.split(',').collect();

    if split.len() != 5 {
        return Err(anyhow!("address must have five comma separated values"));
    }

    // TODO this is bad, index operation can panic
    Ok(Address {
        street_address: split[0].trim().to_string(),
        city: split[1].trim().to_string(),
        state: split[2].trim().to_string(),
        postcode: split[3].trim().to_string(),
        country: split[4].trim().to_string(),
    })
}

fn main() {
    let args = Arguments::parse();

    match args.command {
        Commands::Init {} => {
            let new_phone_book = PhoneBook::new();
            // TODO display a proper error instead of panic on fail
            new_phone_book.save_to_file(OsStr::new(&args.file)).unwrap();
            println!("File created")
        }

        Commands::Add {
            first,
            last,
            phone_number,
            address,
        } => {
            let mut phone_book = PhoneBook::new_from_file(OsStr::new(&args.file)).unwrap();

            phone_book
                .insert_contact(Contact {
                    phone_number,
                    address,
                    first_name: first,
                    last_name: last,
                })
                .unwrap();

            phone_book.save_to_file(OsStr::new(&args.file)).unwrap();
            println!("Contact saved")
        }

        Commands::Update {
            first,
            last,
            phone_number,
            address,
        } => {
            let mut phone_book = PhoneBook::new_from_file(OsStr::new(&args.file)).unwrap();
            let mut existing_contact = phone_book.find_phone_number(phone_number).unwrap().clone();

            match first {
                Some(name) => existing_contact.first_name = name,
                _ => {}
            }
            match last {
                Some(name) => existing_contact.last_name = name,
                _ => {}
            }
            match address {
                Some(add) => existing_contact.address = Some(add),
                _ => {}
            }

            phone_book.replace_contact(existing_contact).unwrap();
            phone_book.save_to_file(OsStr::new(&args.file)).unwrap();
            println!("Contact updated")
        }

        Commands::Delete { phone_number } => {
            let mut phone_book = PhoneBook::new_from_file(OsStr::new(&args.file)).unwrap();
            phone_book.delete_contact(phone_number).unwrap();
            phone_book.save_to_file(OsStr::new(&args.file)).unwrap();
            println!("Contact deleted")
        }

        Commands::Search(search) => match search.command {
            SearchCommands::Name { first, last } => {
                if first == None && last == None {
                    println!("must provide at least one search value");
                    exit(1)
                }

                let phone_book = PhoneBook::new_from_file(OsStr::new(&args.file)).unwrap();

                let search_results = phone_book.find_name(first, last);

                if search_results.len() == 0 {
                    println!("didn't find anything");
                    exit(0)
                }

                for result in search_results {
                    println!("{:?}", result)
                }
            }
            SearchCommands::Phone { phone_number } => {
                let phone_book = PhoneBook::new_from_file(OsStr::new(&args.file)).unwrap();
                match phone_book.find_phone_number(phone_number) {
                    Ok(result) => println!("{:?}", result),
                    _ => println!("didn't find anything"),
                }
            }
            SearchCommands::City { city } => {
                let phone_book = PhoneBook::new_from_file(OsStr::new(&args.file)).unwrap();
                let search_results = phone_book.find_city(city);

                if search_results.len() == 0 {
                    println!("didn't find anything");
                    exit(0)
                }

                for result in search_results {
                    println!("{:?}", result)
                }
            }
            SearchCommands::Fuzzy { search } => {
                todo!("implement fuzzy search")
            }
            SearchCommands::Prefix { search } => {
                todo!("implement phone prefix search")
            }
        },
    }
}
