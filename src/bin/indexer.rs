use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use borsh::BorshDeserialize;
use solana_transaction_status::EncodedTransactionWithStatusMeta;
use solana_transaction_status::option_serializer::OptionSerializer;

use vault_backend::idl;

#[derive(Debug)]
pub enum VaultEvent {
    Initialize {
        vault: String,
        owner: String,
    },
    Deposit {
        vault: String,
        user: String,
        amount: u64,
    },
    Withdraw {
        vault: String,
        user: String,
        amount: u64,
    },
    Lock {
        vault: String,
        amount: u64,
    },
    Unlock {
        vault: String,
        amount: u64,
    },
    Transfer {
        from: String,
        to: String,
        amount: u64,
    },
}

fn parse_event(data: &[u8]) -> anyhow::Result<Option<VaultEvent>> {
    if data.len() < 8 {
        return Ok(None);
    }

    match &data[..8] {
        // DepositEvent
        [120, 248, 61, 83, 31, 142, 107, 144] => {
            let ev = idl::DepositEvent::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::Deposit {
                vault: ev.user.to_string(),
                user: ev.user.to_string(),
                amount: ev.amount,
            }))
        }

        // CollateralWithdrawn
        [51, 224, 133, 106, 74, 173, 72, 82] => {
            let ev = idl::CollateralWithdrawn::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::Withdraw {
                vault: ev.vault.to_string(),
                user: ev.user.to_string(),
                amount: ev.amount,
            }))
        }

        _ => Ok(None),
    }
}

pub fn decode_events(tx: &EncodedTransactionWithStatusMeta) -> anyhow::Result<Vec<VaultEvent>> {
    let mut events = vec![];

    let meta = match &tx.meta {
        Some(m) => m,
        None => return Ok(events),
    };

    let logs = match &meta.log_messages {
        OptionSerializer::Some(l) => l,
        _ => return Ok(events),
    };

    for log in logs {
        if let Some(payload) = log.strip_prefix("Program log: ") {
            if let Ok(bytes) = STANDARD.decode(payload) {
                if let Some(ev) = parse_event(&bytes)? {
                    events.push(ev);
                }
            }
        }
    }

    Ok(events)
}

fn main() {
    println!("Indexer binary");
}

