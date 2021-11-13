#![allow(non_snake_case)] // for our name MFEKinit

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use env_logger;
use log;
use xmltree;

use std;
use std::env;
use std::fs;
use std::io::{self, Write};

mod error;

use error::InitError;
use error::InitResult::{self, *};

static AUTHOR: &str = "Fredrick R. Brennan <copypaste@kittens.ph>";
static VERSION: &str = "0.2.0";

fn clap_app() -> App<'static, 'static> {
    App::new("MFEKinit")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(VERSION)
        .author(AUTHOR)
        .subcommand(
            SubCommand::with_name("glif")
                .alias("GLIF")
                .about("Initialize a .glif file")
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
                .about("Initialize a .ufo font")
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
                        .help("Delete existing UFO font if exists")
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
    env_logger::init();

    let argparser = clap_app();
    let matches = argparser.get_matches();
    let res = match matches.subcommand_name() {
        Some("glif") => glif_main(&matches.subcommand_matches("glif").unwrap()),
        Some("ufo") => ufo_main(&matches.subcommand_matches("ufo").unwrap()),
        _ => InitErr(InitError::NoCommand),
    };

    match res {
        InitErr(ref e) => {
            eprintln!("{}", e)
        }
        _ => {}
    }

    eprintln!("{:?}", &res);
    res.into()
}

fn xmlconfig() -> xmltree::EmitterConfig {
    xmltree::EmitterConfig::new()
        .line_separator("\n")
        .perform_indent(true)
}

fn glif_main(matches: &ArgMatches) -> InitResult {
    let mut ret: InitResult = InitErr(InitError::NoCommand);
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
                ret = InitErr(InitError::FailedGlif);
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
    log::warn!(target: "MFEKinit::UFO", "This feature is experimental and doesn't create anything close to a complete UFO!");
    let del = matches.is_present("delete-if-exists");
    let pathbuf: std::path::PathBuf = matches.value_of("OUT").unwrap().to_string().into();
    let ufodiro = pathbuf
        .iter()
        .last()
        .map(|o| o.to_owned())
        .unwrap_or_else(|| std::ffi::OsString::new());
    let ufodir = ufodiro.to_str().unwrap();

    //eprintln!("{:?} {:?} {:?} {:?} {:?}", &pathbuf, del, pathbuf.is_dir(), ufodir.ends_with("ufo"), ufodir.ends_with("ufo3"));
    if del && pathbuf.is_dir() && (ufodir.ends_with("ufo") || ufodir.ends_with("ufo3")) {
        log::warn!("Deleting {:?}", &pathbuf);
        fs::remove_dir_all(&pathbuf).unwrap();
    }
    match fs::create_dir(&pathbuf) {
        Ok(_) => {
            log::info!("Created {:?}", &pathbuf);
        }
        Err(e) => {
            log::error!("Could not create font because: \"{}\"", e);
            return InitErr(InitError::FailedUFO);
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
            return InitErr(InitError::FailedUFO);
        }
    }
    UfoOk(pathbuf)
}
