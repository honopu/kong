use crate::stable_claim::claim_map;
use crate::stable_kong_settings::kong_settings_map;
use crate::stable_request::reply::Reply;
use crate::stable_request::request_map;
use crate::stable_transfer::transfer_map;
use crate::stable_tx::tx_map;

pub fn archive_to_kong_data(request_id: u64) -> Result<(), String> {
    if kong_settings_map::get().archive_to_kong_data {
        let request =
            request_map::get_by_request_id(request_id).ok_or(format!("Failed to archive. request_id #{} not found", request_id))?;
        request_map::archive_to_kong_data(&request)?;
        match request.reply {
            Reply::Swap(ref reply) => {
                for claim_id in reply.claim_ids.iter() {
                    claim_map::archive_claim_to_kong_data(*claim_id);
                }
                for transfer_id_reply in reply.transfer_ids.iter() {
                    transfer_map::archive_transfer_to_kong_data(transfer_id_reply.transfer_id);
                }
                tx_map::archive_tx_to_kong_data(reply.tx_id);
            }
            _ => Err("Invalid reply type".to_string())?,
        }
    }

    Ok(())
}
