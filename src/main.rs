use miette::{IntoDiagnostic, Result};

mod archive;
mod state;
mod types;

use types::{Hashable, OutPoint, Output, ADDRESS_LENGTH};

fn main() -> Result<()> {
    let outpoint = OutPoint {
        number: 0,
        index: 0,
    };

    let output = Output {
        address: [0; ADDRESS_LENGTH],
        value: 0,
    };

    let outpoint_bytes = bincode::serialize(&outpoint).into_diagnostic()?;
    let output_bytes = bincode::serialize(&output).into_diagnostic()?;

    dbg!(outpoint_bytes.len() + output_bytes.len());
    dbg!(hex::encode(outpoint.hash()));
    dbg!(hex::encode(output.hash()));

    Ok(())
}
