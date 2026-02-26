use bzip2::{read::BzDecoder, write::BzEncoder};
use const_format::formatcp;
use rusqlite as sqlite;
use std::{ffi::OsStr, fs::{self, File}, io::{self, Write}, path::{Path, PathBuf}, process::{Command, Stdio}};
use quote::quote;
use proc_macro2::Literal;

const WORKSPACE_PARENT: &'static str = "../..";
const STARMAP_FILE: &'static str =  "crates/solar_sonar/src/generated/starmap.rs";
const SDE_FILE: &'static str = "target/solar_sonar_dev/eve_sde.sqlite";
const SDE_FILE_BZ2: &'static str = "target/solar_sonar_dev/eve_sde.sqlite.bz2";
const SDE_URL: &'static str = "https://media.githubusercontent.com/media/cornmonger/solar_sonar_asssets/refs/heads/dev/thirdparty/fuzzwork/sde/sqlite-latest.sqlite.bz2";
const DEBUG_BUILD_DIR: &'static str = "target/debug/build";
const ESPEAK_BUILD_PREFIX: &'static str = "espeak-rs-sys-";
const ESPEAK_DATA_SUFFIX: &'static str = "out/share/espeak-ng-data";

const CLR: &'static str = "\x1b[0m";
const CYAN: &'static str = "\x1b[36m";
const GREEN: &'static str = "\x1b[32m";
const RED: &'static str = "\x1b[31m";
const YELLOW: &'static str = "\x1b[33m";
const INFO: &'static str = formatcp!("{CYAN}[sonar]{CLR}");
const WARN: &'static str = formatcp!("{YELLOW}[sonar]{CLR}");
const ERR: &'static str = formatcp!("{RED}[sonar]{CLR}");
const MAGENTA: &'static str = "\x1b[35m";

pub fn main() {
    generate_code();
    package_asset_espeak();
}

fn workspace_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(WORKSPACE_PARENT)
        .canonicalize().unwrap()
}

fn generate_code() {
    let sqlite_db = download_sde();
    let sql = sqlite::Connection::open(sqlite_db).unwrap();
    generate_starmap_rs(&sql);
}

fn download_sde() -> PathBuf {
    let filepath = workspace_dir().join(SDE_FILE);
    if filepath.exists() {
        return filepath;
    }


    print!("{INFO} downloading {MAGENTA}SDE database{CLR} ... ");
    let _ = io::stdout().flush();

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

    println!("{GREEN}done{CLR}");
    print!("{INFO} unzipping {MAGENTA}SDE database{CLR} ... ");
    let _ = io::stdout().flush();

    let bz2_file = File::open(&bz2_filepath).unwrap();
    let mut decoder = BzDecoder::new(bz2_file);
    let mut output = File::create(&filepath).unwrap();
    io::copy(&mut decoder, &mut output).unwrap();

    println!("{GREEN}done{CLR}");

    filepath
}

#[derive(Debug)]
struct SolarSystem {
    id: u32,
    name: String,
}

fn generate_starmap_rs(sql: &sqlite::Connection) {
    print!("{INFO} generating {MAGENTA}starmap.rs{CLR} ... ");
    let _ = io::stdout().flush();

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

    println!("{GREEN}done{CLR}");
}

const ASSETS_REPO_DIR: &'static str = "../solar_sonar_assets";
//const ASSETS_REPO_ESPEAK_DIR: &'static str = "thirdparty/espeak";
const ESPEAK_BUILD_CHERRY_FILE: &'static str = "en_dict";
const ASSET_ESPEAK_DATA_BZ2: &'static str = "thirdparty/espeak/espeak_data.tar.bz2";

fn package_asset_espeak() {
    let Ok(build_dir) = workspace_dir().join(DEBUG_BUILD_DIR).canonicalize() else {
        panic!("build dir doesn't exist")
    };

    let assets_repo = workspace_dir().join(ASSETS_REPO_DIR);
    let Ok(assets_repo) = assets_repo.canonicalize() else {
        panic!("assets repo not found: {}", assets_repo.to_string_lossy())
    };

    // repo should be the same branch as this one and be clean
    let git_branch = git_current_branch(&workspace_dir()).expect("git");
    let assets_branch = git_current_branch(&assets_repo).expect("git");
    if git_branch != assets_branch {
        println!("{WARN} {MAGENTA}solar_sonar_assets{CLR} branch is not {YELLOW}{git_branch}{CLR}. skipped");
        return;
    }

    if !is_git_clean(&assets_repo).unwrap() {
        panic!("{ERR} {MAGENTA}solar_sonar_assets{CLR} repository is {RED}unclean{CLR}");
    }

    // find the latest build of espeak
    let espeak_build_data_dir = fs::read_dir(build_dir)
        .unwrap_or_else(|_| panic!("{ERR} build dir does not exist"))
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|p| p.to_str()) else {
                return None;
            };

            match name.starts_with(ESPEAK_BUILD_PREFIX) {
                false => None,
                true => Some((entry, path)),
            }
        })
        .filter_map(|(entry, path)| {
            let Ok(modified) = entry.metadata()
                .and_then(|meta|
                    meta.modified()
                        .and_then(|t| Ok(t))
                )
            else {
                return None
            };
            
            let data_dir = path.join(ESPEAK_DATA_SUFFIX);
            match data_dir.is_dir() {
                true => Some((data_dir, modified)),
                false => None,
            }
        })
        .fold(None, |last, (path, modified)| {
            let Some((last_path, last_modified)) = last else {
                return Some((path, modified))
            };

            match modified > last_modified {
                false => Some((last_path, last_modified)),
                true => Some((path, modified)),
            }
        })
        .and_then(|(path, _)| Some(path))
        .unwrap_or_else(|| panic!("{ERR} espeak build dir not found"));

    let cherry_file = espeak_build_data_dir.join(ESPEAK_BUILD_CHERRY_FILE);
    if !cherry_file.exists() {
        panic!("{ERR} espeak build is incomplete");
    }

    let espeak_asset_bz2 = assets_repo.join(ASSET_ESPEAK_DATA_BZ2).canonicalize()
        .unwrap_or_else(|_| panic!("{ERR} espeak_data.bz2 not found in assets repo"));

    const TMP_ASSET_ESPEAK: &'static str = "target/build/solar_sonar_dev/tmp/asset_espeak";
    let tmp_dir = workspace_dir().join(TMP_ASSET_ESPEAK);
    print!("{INFO} unzipping asset {MAGENTA}espeak_data{CLR} ... ");
    let _ = io::stdout().flush();
    bunzip2_dir(&espeak_asset_bz2, &tmp_dir).unwrap();
    println!("{GREEN}done{CLR}");

    print!("{INFO} comparing {MAGENTA}espeak_data{CLR} asset to build ... ");
    let _ = io::stdout().flush();
    let tmp_espeak_data_dir = tmp_dir.join("espeak_data");
    if !dir_diff::is_different(tmp_espeak_data_dir, &espeak_build_data_dir).unwrap() {
        println!("{GREEN}same{CLR}");
        return;
    }

    println!("{YELLOW}different{CLR}");
    
    const BUILD_ESPEAK_DATA_BZ2: &'static str = "target/build/solar_sonar_dev/espeak_data.tar.bz2";
    let espeak_build_bz2 = workspace_dir().join(BUILD_ESPEAK_DATA_BZ2);
    print!("{INFO} zipping build {MAGENTA}espeak_data{CLR} ... ");
    let _ = io::stdout().flush();
    bzip2_dir(&espeak_build_data_dir, &espeak_build_bz2, "espeak_data").unwrap();
    println!("{GREEN}done{CLR}");

    print!("{INFO} copying build {MAGENTA}espeak_data{CLR} to {CYAN}solar_sonar_assets{CLR} ... ");
    let _ = io::stdout().flush();
    fs::copy(&espeak_build_bz2, &espeak_asset_bz2).unwrap();
    println!("{GREEN}done{CLR}");

    println!("{WARN} repository {MAGENTA}solar_sonar_assets{CLR} is now {YELLOW}staged{CLR}");
}

fn git_output<I, S>(dir: &Path, args: I) -> io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = make_git_command(dir, args, false);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Git unsuccessful"));
    }

    let output = String::try_from(output.stdout).expect("UTF8")
        .trim().to_string();
    Ok(output)
}

/*fn git_code<I, S>(dir: &Path, args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = make_git_command(dir, args, true);
    cmd.status()
}*/

fn make_git_command<I, S>(dir: &Path, args: I, status: bool) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new("git");
    cmd.current_dir(dir);
    cmd.args(args);

    if status {
        cmd.stdout(Stdio::null());
    }
    
    cmd
}


fn git_current_branch(dir: &Path) -> io::Result<String> {
    git_output(dir, ["branch", "--show-current"])
}

fn is_git_clean(dir: &Path) -> io::Result<bool> {
    let output = git_output(dir, ["status", "--porcelain"])?;
    Ok(output.is_empty())
}

fn bzip2_dir(input_dir: &Path, output_file: &Path, prefix: &str) -> std::io::Result<()> {
    if let Some(parent) = output_file.parent() && !parent.is_dir() {
        fs::create_dir_all(parent).unwrap();
    }

    let file = File::create(output_file)?;
    let encoder = BzEncoder::new(file, bzip2::Compression::best());
    let mut archive = tar::Builder::new(encoder);
    archive.append_dir_all(prefix, input_dir)?;
    let encoder = archive.into_inner()?;
    encoder.finish()?;
    Ok(())
}

fn bunzip2_dir(input_file: &Path, output_dir: &Path) -> std::io::Result<()> {
    if let Some(parent) = output_dir.parent() && !parent.is_dir() {
        fs::create_dir_all(parent).unwrap();
    }

    let file = File::open(input_file)?;
    let decoder = BzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(output_dir)?;
    Ok(())
}
