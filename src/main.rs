#![allow(non_snake_case)] // for our name MFEKinit

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use xmltree;

use std::fs;
use std::io::{self, Write};

mod error;

use error::InitError;

static AUTHOR: &str = "Fredrick R. Brennan <copypaste@kittens.ph>";
static VERSION: &str = "0.0.0";

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
                        .help("Name of the glyph")
                        .required(true),
                )
                .arg(
                    Arg::with_name("ENCODING")
                        .help("Unicode encoding of the glyph")
                        .required(true),
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
                        .short("dD")
                        .required(false)
                        .takes_value(false),
                ),
        )
}

fn main() -> Result<(), InitError> {
    let argparser = clap_app();
    let matches = argparser.get_matches();
    let res = match matches.subcommand_name() {
        Some("glif") => glif_main(&matches.subcommand_matches("glif").unwrap()),
        Some("ufo") => ufo_main(&matches.subcommand_matches("ufo").unwrap()),
        _ => Err(InitError::NoCommand),
    };

    match res {
        Err(ref e) => {eprintln!("{}", e)},
        _ => {}
    }

    res
}

fn xmlconfig() -> xmltree::EmitterConfig {
    xmltree::EmitterConfig::new()
        .line_separator("\n")
        .perform_indent(true)
}

fn glif_main(matches: &ArgMatches) -> Result<(), InitError> {
    let mut ret = Ok(());

    let mut root = xmltree::Element::new("glyph");
    root.attributes.insert(
        "name".to_string(),
        matches.value_of("GLYPHNAME").unwrap().to_string(),
    );
    root.attributes
        .insert("format".to_string(), "2".to_string());
    let mut advance = xmltree::Element::new("advance");
    advance
        .attributes
        .insert("width".to_string(), "0".to_string());
    let mut unicode = xmltree::Element::new("unicode");
    unicode.attributes.insert(
        "hex".to_string(),
        matches.value_of("ENCODING").unwrap().to_string(),
    );
    let mut outline = xmltree::Element::new("outline");

    root.children.push(xmltree::XMLNode::Element(advance));
    root.children.push(xmltree::XMLNode::Element(unicode));
    root.children.push(xmltree::XMLNode::Element(outline));
    root.children
        .push(xmltree::XMLNode::Comment(String::from("<MFEK></MFEK>")));

    let outbuf: Option<Box<dyn Write>> = match matches.value_of("OUTFILE") {
        None => Some(Box::new(io::stdout()) as Box<dyn Write>),
        Some(fname) => fs::File::create(fname)
            .map(|f| Some(Box::new(f) as Box<dyn Write>))
            .unwrap_or_else(|_| {
                ret = Err(InitError::FailedGlif);
                None
            }),
    };

    outbuf.map(|o| {
        root.write_with_config(o, xmlconfig()).unwrap();
    });
    ret
}

fn ufo_main(matches: &ArgMatches) -> Result<(), InitError> {
    match fs::create_dir(matches.value_of("OUT").unwrap()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Could not create font because: \"{}\"", e);
            return Err(InitError::FailedUFO);
        }
    }
    Ok(())
}
