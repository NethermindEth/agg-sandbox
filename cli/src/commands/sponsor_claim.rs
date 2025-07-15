use anyhow::Result;
pub async fn handle_sponsor_claim(deposit: u32, l2_from: u32) -> Result<()> {
    println!("(stub) sponsor-claim deposit={deposit} l2_from={l2_from}");
    Ok(())
}