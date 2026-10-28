use anyhow::Result;
use clap::Parser;
use protobuf::Message;
use scip::types::symbol_information::Kind;
use scip::types::Index;
use scip::types::SymbolInformation;
use scip::types::SymbolRole;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(about = "Detect unused types in a SCIP file")]
struct Args {
    #[clap(short, long)]
    workspace: PathBuf,

    /// The generated SCIP file
    #[clap(short, long)]
    index: PathBuf,

    /// The crate to filter results on. Filters using starts_with.
    #[clap(short, long)]
    filter: String,
}

struct Symbol<'a> {
    document: &'a scip::types::Document,
    occurrence: &'a scip::types::Occurrence,
}

fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let index = {
        let reader = std::fs::File::open(&args.index)?;
        let mut reader = std::io::BufReader::new(reader);
        Index::parse_from_reader(&mut reader)?
    };

    let mut symbol_defs: HashMap<&String, Symbol> = HashMap::default();
    let mut structs = HashMap::<&String, &SymbolInformation>::default();

    for doc in &index.documents {
        for s in &doc.symbols {
            let Ok(kind) = s.kind.enum_value() else {
                continue;
            };

            match kind {
                Kind::Struct => {
                    structs.insert(&s.symbol, s);
                }

                _ => {
                    // ignore
                }
            }
        }
    }

    for doc in &index.documents {
        let mut not_counting_as_use_line: HashSet<&i32> = HashSet::default();

        for o in &doc.occurrences {
            // Rust specific: do not count TryFrom or From as a use
            match o.symbol.as_str() {
                "rust-analyzer cargo core https://github.com/rust-lang/rust/library/core convert/TryFrom#"
                | "rust-analyzer cargo core https://github.com/rust-lang/rust/library/core convert/From#" =>
                {
                    not_counting_as_use_line.insert(&o.range[0]);
                    continue;
                }

                _ => {}
            }

            let si = scip::symbol::parse_symbol(&o.symbol).unwrap();

            for descriptor in si.descriptors {
                if descriptor.name.starts_with("From<crate")
                    | descriptor.name.starts_with("TryFrom<crate")
                {
                    not_counting_as_use_line.insert(&o.range[0]);
                    continue;
                }
            }

            // use
            if (o.symbol_roles & SymbolRole::Definition as i32) == 0
                && !not_counting_as_use_line.contains(&o.range[0])
            {
                structs.remove(&o.symbol);
            }

            // def
            if (o.symbol_roles & SymbolRole::Definition as i32) > 0 {
                symbol_defs.insert(
                    &o.symbol,
                    Symbol {
                        document: doc,
                        occurrence: o,
                    },
                );
            }
        }
    }

    // Any symbols left in structs are not used

    let crate_filter = format!("rust-analyzer cargo {}", args.filter);

    for symbol in structs.keys() {
        if !symbol.starts_with(&crate_filter) {
            continue;
        }

        let Some(Symbol {
            document,
            occurrence,
        }) = symbol_defs.get(symbol)
        else {
            eprintln!("could not find def for {}", symbol);
            continue;
        };

        println!(
            "{} | {} | {:?}",
            symbol, document.relative_path, occurrence.range
        );
    }

    Ok(())
}
