use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use borsh::BorshDeserialize;
use solana_transaction_status::EncodedTransactionWithStatusMeta;

use crate::idl;

#[derive(Debug)]
pub enum VaultEvent {
    VaultAuthorityInitialized {
        admin: String,
    },
    ProgramAuthorized {
        program_id: String,
    },
    VaultInitialized {
        vault: String,
        owner: String,
        mint: String,
        timestamp: i64,
    },
    Deposit {
        user: String,
        amount: u64,
        new_balance: u64,
        timestamp: i64,
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

pub fn decode_events(tx: &EncodedTransactionWithStatusMeta) -> anyhow::Result<Vec<VaultEvent>> {
    let mut events = vec![];

    let meta = match &tx.meta {
        Some(m) => m,
        None => return Ok(events),
    };

    use solana_transaction_status::option_serializer::OptionSerializer;

    let logs = match &meta.log_messages {
        OptionSerializer::Some(l) => l,
        _ => return Ok(events),
    };

    for log in logs {
        // Anchor event logs
        if let Some(payload) = log.strip_prefix("Program log: ") {
            // Avoid decoding non-base64 logs
            if !payload
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
            {
                continue;
            }

            if let Ok(bytes) = STANDARD.decode(payload) {
                if let Some(event) = parse_event(&bytes)? {
                    events.push(event);
                }
            }
        }
    }

    Ok(events)
}

fn parse_event(data: &[u8]) -> anyhow::Result<Option<VaultEvent>> {
    if data.len() < 8 {
        return Ok(None);
    }

    match &data[..8] {
        // VaultAuthorityInitialized
        [95, 255, 252, 53, 25, 33, 57, 40] => {
            let ev = idl::VaultAuthorityInitialized::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::VaultAuthorityInitialized {
                admin: ev.admin.to_string(),
            }))
        }

        // ProgramAuthorized
        [59, 38, 123, 101, 35, 35, 172, 29] => {
            let ev = idl::ProgramAuthorized::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::ProgramAuthorized {
                program_id: ev.program_id.to_string(),
            }))
        }

        // VaultInitialized
        [180, 43, 207, 2, 18, 71, 3, 75] => {
            let ev = idl::VaultInitialized::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::VaultInitialized {
                vault: ev.vault.to_string(),
                owner: ev.owner.to_string(),
                mint: ev.mint.to_string(),
                timestamp: ev.timestamp,
            }))
        }

        // DepositEvent
        [120, 248, 61, 83, 31, 142, 107, 144] => {
            let ev = idl::DepositEvent::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::Deposit {
                user: ev.user.to_string(),
                amount: ev.amount,
                new_balance: ev.new_balance,
                timestamp: ev.timestamp,
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

        // CollateralLocked
        [185, 146, 119, 8, 41, 179, 88, 96] => {
            let ev = idl::CollateralLocked::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::Lock {
                vault: ev.vault.to_string(),
                amount: ev.amount,
            }))
        }

        // CollateralUnlocked
        [195, 248, 152, 155, 116, 178, 189, 221] => {
            let ev = idl::CollateralUnlocked::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::Unlock {
                vault: ev.vault.to_string(),
                amount: ev.amount,
            }))
        }

        // CollateralTransferred
        [119, 180, 79, 171, 178, 67, 120, 237] => {
            let ev = idl::CollateralTransferred::try_from_slice(&data[8..])?;
            Ok(Some(VaultEvent::Transfer {
                from: ev.from.to_string(),
                to: ev.to.to_string(),
                amount: ev.amount,
            }))
        }

        _ => Ok(None),
    }
}
