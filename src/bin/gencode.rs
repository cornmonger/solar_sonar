use rusqlite as sqlite;
use clap::Parser;
use std::{fs, path::{Path, PathBuf}};
use quote::quote;
use proc_macro2::Literal;

/// Generates the project src/generated code-base.
#[derive(Debug, clap::Parser)]
struct Cli {
    /// Path to the FuzzWorks SDE .sqlite file
    /// Download from: https://www.fuzzwork.co.uk/dump
    sqlite_db: PathBuf,
}

pub fn main() {
    let cli = Cli::parse();
    let sql = sqlite::Connection::open(cli.sqlite_db)
        .expect("Unable to open SQLite DB");

    generate_starmap_rs(&sql);
}

#[derive(Debug)]
struct SolarSystem {
    id: u32,
    name: String,
}

fn generate_starmap_rs(sql: &sqlite::Connection) {
    let num_systems = sql.query_one(r#"
            SELECT COUNT(solarSystemID) FROM mapSolarSystems
        "#, [], |row| {
            let count: u32 = row.get_unwrap(0);
            Ok(count as usize)
        })
        .expect("SQL count query failed");

    let mut qry = sql.prepare(r#"
            SELECT solarSystemID, solarSystemName
            FROM mapSolarSystems
            ORDER BY solarSystemID ASC
        "#)
        .expect("Invalid query");

    let rslt = qry.query_map([], |row| {
        Ok(SolarSystem {
            id: row.get(0).expect("id"),
            name: row.get(1).expect("name"),
        })
    }).expect("Unable to fetch solar systems");

    let mut jumps_qry = sql.prepare(r#"
        SELECT toSolarSystemID FROM mapSolarSystemJumps
        WHERE fromSolarSystemID = ?
        ORDER BY toSolarSystemID ASC;
    "#)
    .expect("Invalid jump query");

    let mut systems = Vec::with_capacity(num_systems);
    for sys in rslt {
        let sys = sys.expect("sys");
        let jumps_rslt = jumps_qry.query_map([sys.id], |row| {
            let id: u32 = row.get(0).expect("to sys id");
            Ok(id)
        }).expect("bad jmp result");

        let gates: Vec<_> = jumps_rslt.into_iter().map(|id| {
            let id = Literal::u32_unsuffixed(id.expect("id"));
            id
        }).collect();

        let id = Literal::u32_unsuffixed(sys.id);
        let name = sys.name;
        systems.push(quote!{
            SolarSystem {
                id: #id,
                name: #name,
                gates: &[ #(#gates),* ]
            },
        });
    }

    let src = quote!{
        use crate::*;
        
        pub const STAR_MAP: StarMap = StarMap {
            systems: &[
                #(#systems)*
            ],
        };
    };

    let output_file = Path::new(std::env!("CARGO_MANIFEST_DIR"))
        .join("src/generated/starmap.rs");

    let tokens = src.to_string();
    fs::write(&output_file, &tokens).expect("file write");
    let parsed = syn::parse_file(&tokens).unwrap();
    let formatted_code = prettyplease::unparse(&parsed);
    fs::write(output_file, formatted_code).expect("file write");
}
