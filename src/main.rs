#![allow(non_snake_case)] // for our name MFEKinit

use bak::BakExtension;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use env_logger;
use local_encoding::Encoding as LocalEncoding;
use local_encoding::Encoder as _;
use log;
use xmltree;

use std;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path;

mod error;
mod util;
use util::FilesToWrite;

use error::InitError;
use error::InitResult::{self, *};

static AUTHOR: &str = "Fredrick R. Brennan <copypaste@kittens.ph>";
static VERSION: &str = env!("CARGO_PKG_VERSION");

fn clap_app() -> App<'static, 'static> {
    App::new("MFEKinit")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .version(VERSION)
        .author(AUTHOR)
        .subcommand(
            SubCommand::with_name("glif")
                .alias("GLIF")
                .about("Initialize an empty .glif file")
                .version(VERSION)
                .author(AUTHOR)
                .arg(
                    Arg::with_name("GLYPHNAME")
                        .short("n")
                        .long("name")
                        .takes_value(true)
                        .help("Name of the glyph")
                )
                .arg(
                    Arg::with_name("ENCODING")
                        .short("e")
                        .long("encoding")
                        .takes_value(true)
                        .help("Unicode encoding of the glyph")
                )
                .arg(
                    Arg::with_name("WIDTH")
                        .short("w")
                        .long("width")
                        .takes_value(true)
                        .help("Width of the glyph")
                )
                .arg(
                    Arg::with_name("HEIGHT")
                        .short("H")
                        .long("height")
                        .takes_value(true)
                        .help("Height of the glyph (note: you probably don't want to set this, it's used for vertical kerning e.g. for CJK, not to define ascender height)")
                )
                .arg(
                    Arg::with_name("OUTFILE")
                        .help("Output filename")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("ufo")
                .alias("UFO")
                .about("Initialize an empty .ufo font (Mary, Mary, spec-contrary)")
                .version(VERSION)
                .author(AUTHOR)
                .arg(
                    Arg::with_name("OUT")
                        .help("Output .ufo font")
                        .required(true),
                )
                .arg(
                    Arg::with_name("delete-if-exists")
                        .long("delete-if-exists")
                        .help("Delete existing UFO font if exists (by default, UFO is moved out of the way)")
                        .short("D")
                        .required(false)
                        .takes_value(false),
                ),
        )
}

fn main() -> Result<(), InitError> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }
    mfek_ipc::display_header("init");
    env_logger::init();

    let argparser = clap_app();
    let matches = argparser.get_matches();
    let res = match matches.subcommand_name() {
        Some("glif") => glif_main(&matches.subcommand_matches("glif").unwrap()),
        Some("ufo") => ufo_main(&matches.subcommand_matches("ufo").unwrap()),
        _ => Error(InitError::NoCommand),
    };

    match res {
        Error(ref e) => {
            log::error!("{}", e)
        }
        _ => {}
    }

    res.into()
}

fn xmlconfig() -> xmltree::EmitterConfig {
    xmltree::EmitterConfig::new()
        .line_separator("\n")
        .perform_indent(true)
}

fn glif_main(matches: &ArgMatches) -> InitResult {
    let mut ret: InitResult = Error(InitError::NoCommand);
    let glyphname = match matches.value_of("GLYPHNAME") {
        Some(g) => g.strip_suffix(".glif").unwrap_or(g),
        None => "glyph",
    };
    let width = matches.value_of("WIDTH").unwrap_or("0");
    let height = matches.value_of("HEIGHT");

    let mut root = xmltree::Element::new("glyph");
    root.attributes
        .insert("name".to_string(), glyphname.to_string());
    root.attributes
        .insert("format".to_string(), "2".to_string());
    let mut advance = xmltree::Element::new("advance");
    advance
        .attributes
        .insert("width".to_string(), width.to_string());

    match height {
        Some(h) => {
            advance
                .attributes
                .insert("height".to_string(), h.to_string());
        }
        None => (),
    };

    let outline = xmltree::Element::new("outline");

    root.children.push(xmltree::XMLNode::Element(advance));
    root.children.push(xmltree::XMLNode::Element(outline));

    match matches.value_of("ENCODING") {
        Some(encoding) => {
            let mut unicode = xmltree::Element::new("unicode");
            unicode
                .attributes
                .insert("hex".to_string(), encoding.to_string());
            root.children.push(xmltree::XMLNode::Element(unicode));
        }
        None => (),
    }

    if let Some(fname) = matches.value_of("OUTFILE") {
        fs::File::create(fname)
            .map(|f| {
                ret = GlifOk(fname.to_string(), Box::new(f) as Box<dyn Write>);
            })
            .unwrap_or_else(|_| {
                ret = Error(InitError::FailedGlif);
            });
    } else {
        ret = GlifStdoutOk(Box::new(io::stdout()) as Box<dyn Write>);
    }

    if let GlifOk(_, ref mut o) = ret {
        root.write_with_config(o, xmlconfig()).unwrap();
    }
    ret
}

fn ufo_main(matches: &ArgMatches) -> InitResult {
    log::warn!(target: "MFEKinit::UFO", "This feature is experimental and doesn't create a complete UFO!\n\
                        Third party tools are still unlikely to validate MFEKinit-produced UFO's due to missing files;\n\
                        see linebender/norad№242 (GitHub).");
    let del = matches.is_present("delete-if-exists");
    let pathbuf: std::path::PathBuf = matches.value_of("OUT").unwrap().to_string().into();
    let ufodiro = pathbuf
        .iter()
        .last()
        .map(|o| o.to_owned())
        .unwrap_or_else(|| std::ffi::OsString::new());
    let ufodir = ufodiro.to_str().unwrap();

    let bak_ext = BakExtension::new_format_str("{++}.bak.ufo".into()).no_prepend_period_to_n();
    if pathbuf.is_dir() && (ufodir.ends_with("ufo") || ufodir.ends_with("ufo3")) {
        if del {
            fs::remove_dir_all(&pathbuf).unwrap();
            log::warn!("Deleted {:?}, as requested!!", &pathbuf);
        } else {
            let moved = bak::move_aside_with_extension(&pathbuf, &bak_ext).unwrap();
            log::error!("{:?} existed — so we moved it aside, to {:?}", &pathbuf, &moved);
        }
    }
    match fs::create_dir(&pathbuf) {
        Ok(_) => {
            log::info!("Created {:?}", &pathbuf);
        }
        Err(e) => {
            log::error!("Could not create font because: \"{}\"", e);
            return Error(InitError::FailedUFO);
        }
    }
    let glyphsdir = pathbuf.clone().join("glyphs");
    match fs::create_dir(&glyphsdir) {
        Ok(_) => {
            log::info!("Created {:?}", &glyphsdir);
        }
        Err(e) => {
            log::error!(
                "Could not create font's glyphs dir because because: \"{}\"",
                e
            );
            return Error(InitError::FailedUFO);
        }
    }

    fn vec_u8_to_pb(dir: impl Into<path::PathBuf>, gf: &[u8]) -> path::PathBuf {
        dir.into().join(path::PathBuf::from(LocalEncoding::OEM.to_string(gf).unwrap()))
    }
    let glyphsdir_files: FilesToWrite = util::GLYPHSDIR_WRITTEN.into_iter().map(|(gf, c)| (true, pathbuf.clone().join(vec_u8_to_pb("glyphs", gf)), *c)).collect();
    let topdir_files: FilesToWrite = util::TOPLEVEL_WRITTEN.into_iter().map(|(gf, c)| (false, vec_u8_to_pb(&pathbuf, gf), *c)).collect();

    for (check_in_glyphs_dir, filename, contents) in glyphsdir_files.into_iter().chain(topdir_files.into_iter()) {
        assert!(!check_in_glyphs_dir || (filename.parent().unwrap().file_name().unwrap().to_string_lossy().starts_with("glyphs") && filename.parent().unwrap().is_dir()));
        let mut file = match fs::File::create(&filename) {
            Ok(f) => f,
            Err(e) => {
                log::error!("Could not create glyph-level file {:?} because: \"{}\"", filename, e);
                return Error(InitError::FailedUFO)
            }
        };
        match file.write(contents) {
            Ok(len) => log::debug!("Created {:?} (len {}) ({}…)", filename, len, LocalEncoding::OEM.to_string(&contents[0..usize::min(256, contents.len())]).unwrap()),
            Err(e) => {
                log::error!("Could not create glyph-level file {:?} because: \"{}\"", filename, e);
                return Error(InitError::FailedUFO)
            }
        }
    }
    UfoOk(pathbuf)
}
