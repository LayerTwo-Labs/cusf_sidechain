use miette::Result;
use serde::{Deserialize, Serialize};

use crate::types::ADDRESS_LENGTH;

#[derive(Serialize, Deserialize)]
struct Authorization {}

fn is_authorized(
    authorizations: &[Authorization],
    addresses: &[[u8; ADDRESS_LENGTH]],
) -> Result<()> {
    todo!();
}
