use bzip2::read::BzDecoder;
use rusqlite as sqlite;
use std::{fs::{self, File}, io, path::{Path, PathBuf}};
use quote::quote;
use proc_macro2::Literal;

const WORKSPACE_PARENT: &'static str = "../..";
const STARMAP_FILE: &'static str =  "crates/solar_sonar/src/generated/starmap.rs";
const SDE_FILE: &'static str = "target/solar_sonar_dev/eve_sde.sqlite";
const SDE_FILE_BZ2: &'static str = "target/solar_sonar_dev/eve_sde.sqlite.bz2";
const SDE_URL: &'static str = "https://media.githubusercontent.com/media/cornmonger/solar_sonar_asssets/refs/heads/dev/thirdparty/fuzzwork/sde/sqlite-latest.sqlite.bz2";

pub fn main() {
    let sqlite_db = download_sde();
    let sql = sqlite::Connection::open(sqlite_db).unwrap();
    generate_starmap_rs(&sql);
}

fn workspace_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(WORKSPACE_PARENT)
        .canonicalize().unwrap()
}

fn download_sde() -> PathBuf {
    let filepath = workspace_dir().join(SDE_FILE);
    if filepath.exists() {
        return filepath;
    }

    let bz2_filepath = workspace_dir().join(SDE_FILE_BZ2);
    let dir = bz2_filepath.parent().unwrap();
    if !dir.exists() {
        fs::create_dir_all(dir).unwrap();
    }
    
    let mut resp = reqwest::blocking::get(SDE_URL)
        .unwrap()
        .error_for_status().unwrap();
    let mut dest = File::create(&bz2_filepath).unwrap();
    io::copy(&mut resp, &mut dest).unwrap();

    let bz2_file = File::open(&bz2_filepath).unwrap();
    let mut decoder = BzDecoder::new(bz2_file);
    let mut output = File::create(&filepath).unwrap();
    io::copy(&mut decoder, &mut output).unwrap();

    filepath
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


    let output_file = workspace_dir().join(STARMAP_FILE);
    let tokens = src.to_string();
    fs::write(&output_file, &tokens).expect("file write");
    let parsed = syn::parse_file(&tokens).unwrap();
    let formatted_code = prettyplease::unparse(&parsed);
    fs::write(output_file, formatted_code).expect("file write");
}
