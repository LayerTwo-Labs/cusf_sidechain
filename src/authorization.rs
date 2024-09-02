use cusf_sidechain_types::ADDRESS_LENGTH;
use miette::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Authorization {}

fn is_authorized(
    authorizations: &[Authorization],
    addresses: &[[u8; ADDRESS_LENGTH]],
) -> Result<()> {
    todo!();
}
