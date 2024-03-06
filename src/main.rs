mod authorization;
mod net;
mod node;
mod server;
mod state;
mod types;

use miette::{IntoDiagnostic, Result};
use types::{Hashable, OutPoint, Output, ADDRESS_LENGTH};

fn main() -> Result<()> {
    let outpoint = OutPoint {
        number: 0 | 1u64.rotate_right(1),
        index: 0,
    };

    let output = Output::Regular {
        address: [0; ADDRESS_LENGTH],
        value: 0,
    };

    let outpoint_bytes = bincode::serialize(&outpoint).into_diagnostic()?;
    let output_bytes = bincode::serialize(&output).into_diagnostic()?;

    println!("{outpoint}");
    dbg!(outpoint_bytes.len());
    dbg!(output_bytes.len());
    dbg!(hex::encode(outpoint.hash()));
    dbg!(hex::encode(output.hash()));
    dbg!(hex::encode(output.address()));

    Ok(())
}
