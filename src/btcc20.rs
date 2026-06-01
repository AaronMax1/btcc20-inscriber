use super::*;
use serde::{Deserializer, Serializer};

pub const PROTOCOL: &str = "btcc-20";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Operation {
  Deploy {
    tick: String,
    max: String,
    lim: String,
    dec: u8,
  },
  Mint {
    tick: String,
    amt: String,
  },
  Transfer {
    tick: String,
    amt: String,
  },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Payload {
  pub protocol: String,
  pub operation: Operation,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Ledger {
  pub tokens: BTreeMap<String, Token>,
  pub balances: BTreeMap<String, BTreeMap<String, Balance>>,
  pub transfers: BTreeMap<InscriptionId, Transferable>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Token {
  pub tick: String,
  #[serde(deserialize_with = "deserialize_u128")]
  #[serde(serialize_with = "serialize_u128_string")]
  pub max: u128,
  #[serde(deserialize_with = "deserialize_u128")]
  #[serde(serialize_with = "serialize_u128_string")]
  pub lim: u128,
  pub dec: u8,
  #[serde(deserialize_with = "deserialize_u128")]
  #[serde(serialize_with = "serialize_u128_string")]
  pub minted: u128,
  pub deployer: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Balance {
  #[serde(deserialize_with = "deserialize_u128")]
  #[serde(serialize_with = "serialize_u128_string")]
  pub available: u128,
  #[serde(deserialize_with = "deserialize_u128")]
  #[serde(serialize_with = "serialize_u128_string")]
  pub transferable: u128,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Transferable {
  pub tick: String,
  #[serde(deserialize_with = "deserialize_u128")]
  #[serde(serialize_with = "serialize_u128_string")]
  pub amount: u128,
  pub owner: String,
  pub spent: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Event {
  pub valid: bool,
  pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawPayload {
  p: String,
  op: String,
  tick: String,
  max: Option<String>,
  lim: Option<String>,
  dec: Option<String>,
  amt: Option<String>,
}

fn serialize_u128_string<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&value.to_string())
}

fn deserialize_u128<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
  D: Deserializer<'de>,
{
  let value = serde_json::Value::deserialize(deserializer)?;
  match value {
    serde_json::Value::String(value) => value.parse().map_err(serde::de::Error::custom),
    serde_json::Value::Number(value) => value
      .as_u64()
      .map(u128::from)
      .ok_or_else(|| serde::de::Error::custom("invalid u128 number")),
    _ => Err(serde::de::Error::custom("expected u128 string or number")),
  }
}

impl Ledger {
  pub fn apply_inscription(
    &mut self,
    inscription_id: InscriptionId,
    owner: impl Into<String>,
    payload: Payload,
  ) -> Event {
    let owner = owner.into();

    match payload.operation {
      Operation::Deploy {
        tick,
        max,
        lim,
        dec,
      } => {
        if self.tokens.contains_key(&tick) {
          return Event::invalid("ticker already deployed");
        }

        let Some(max) = parse_amount(&max, dec) else {
          return Event::invalid("invalid max");
        };
        let Some(lim) = parse_amount(&lim, dec) else {
          return Event::invalid("invalid lim");
        };
        if lim > max {
          return Event::invalid("lim exceeds max");
        }

        self.tokens.insert(
          tick.clone(),
          Token {
            tick,
            max,
            lim,
            dec,
            minted: 0,
            deployer: owner,
          },
        );

        Event::valid()
      }
      Operation::Mint { tick, amt } => {
        let Some(token) = self.tokens.get_mut(&tick) else {
          return Event::invalid("ticker not deployed");
        };
        let Some(amount) = parse_amount(&amt, token.dec) else {
          return Event::invalid("invalid amount");
        };
        if amount > token.lim {
          return Event::invalid("mint exceeds limit");
        }
        if token.minted.saturating_add(amount) > token.max {
          return Event::invalid("mint exceeds max");
        }

        token.minted += amount;
        self.balance_mut(&tick, &owner).available += amount;

        Event::valid()
      }
      Operation::Transfer { tick, amt } => {
        let Some(token) = self.tokens.get(&tick) else {
          return Event::invalid("ticker not deployed");
        };
        let Some(amount) = parse_amount(&amt, token.dec) else {
          return Event::invalid("invalid amount");
        };

        let balance = self.balance_mut(&tick, &owner);
        if balance.available < amount {
          return Event::invalid("insufficient available balance");
        }

        balance.available -= amount;
        balance.transferable += amount;
        self.transfers.insert(
          inscription_id,
          Transferable {
            tick,
            amount,
            owner,
            spent: false,
          },
        );

        Event::valid()
      }
    }
  }

  pub fn apply_transfer_spend(
    &mut self,
    inscription_id: InscriptionId,
    new_owner: impl Into<String>,
  ) -> Event {
    let new_owner = new_owner.into();
    let Some(transfer) = self.transfers.get_mut(&inscription_id) else {
      return Event::invalid("transfer inscription not found");
    };
    if transfer.spent {
      return Event::invalid("transfer inscription already spent");
    }

    let old_owner = transfer.owner.clone();
    let tick = transfer.tick.clone();
    let amount = transfer.amount;
    transfer.spent = true;

    let old_balance = self.balance_mut(&tick, &old_owner);
    old_balance.transferable = old_balance.transferable.saturating_sub(amount);
    self.balance_mut(&tick, &new_owner).available += amount;

    Event::valid()
  }

  pub fn balance(&self, tick: &str, owner: &str) -> Balance {
    self
      .balances
      .get(tick)
      .and_then(|balances| balances.get(owner))
      .copied()
      .unwrap_or_default()
  }

  fn balance_mut(&mut self, tick: &str, owner: &str) -> &mut Balance {
    self
      .balances
      .entry(tick.into())
      .or_default()
      .entry(owner.into())
      .or_default()
  }
}

impl Event {
  fn valid() -> Self {
    Self {
      valid: true,
      reason: None,
    }
  }

  fn invalid(reason: impl Into<String>) -> Self {
    Self {
      valid: false,
      reason: Some(reason.into()),
    }
  }
}

impl Payload {
  pub fn from_inscription(inscription: &Inscription) -> Result<Option<Self>> {
    if let Some(metaprotocol) = inscription.metaprotocol()
      && metaprotocol != PROTOCOL
    {
      return Ok(None);
    }

    let Some(body) = inscription.body() else {
      return Ok(None);
    };

    let raw = match serde_json::from_slice::<RawPayload>(body) {
      Ok(raw) => raw,
      Err(_) => return Ok(None),
    };

    if raw.p != PROTOCOL {
      return Ok(None);
    }

    Ok(Some(Self::try_from(raw)?))
  }

  pub fn from_json_slice(slice: &[u8]) -> Result<Self> {
    Self::try_from(serde_json::from_slice::<RawPayload>(slice)?)
  }
}

impl TryFrom<RawPayload> for Payload {
  type Error = Error;

  fn try_from(raw: RawPayload) -> Result<Self> {
    ensure!(raw.p == PROTOCOL, "protocol must be {PROTOCOL}");

    let tick = normalize_tick(&raw.tick)?;
    let operation = match raw.op.as_str() {
      "deploy" => {
        let max = valid_decimal_string(raw.max.as_deref().context("missing max")?)?;
        let lim = valid_decimal_string(raw.lim.as_deref().context("missing lim")?)?;
        let dec = match raw.dec {
          Some(dec) => dec.parse::<u8>().context("invalid dec")?,
          None => 18,
        };

        ensure!(dec <= 18, "dec must be between 0 and 18");

        Operation::Deploy {
          tick,
          max,
          lim,
          dec,
        }
      }
      "mint" => Operation::Mint {
        tick,
        amt: valid_decimal_string(raw.amt.as_deref().context("missing amt")?)?,
      },
      "transfer" => Operation::Transfer {
        tick,
        amt: valid_decimal_string(raw.amt.as_deref().context("missing amt")?)?,
      },
      op => bail!("unsupported btcc-20 operation `{op}`"),
    };

    Ok(Self {
      protocol: PROTOCOL.into(),
      operation,
    })
  }
}

fn normalize_tick(tick: &str) -> Result<String> {
  let tick = tick.trim().to_lowercase();
  ensure!(tick.chars().count() == 4, "tick must be 4 characters");
  ensure!(
    tick.chars().all(|c| c.is_ascii_alphanumeric()),
    "tick must be ASCII alphanumeric"
  );
  Ok(tick)
}

fn valid_decimal_string(value: &str) -> Result<String> {
  let value = value.trim();
  ensure!(!value.is_empty(), "amount is empty");
  ensure!(
    value.chars().all(|c| c.is_ascii_digit() || c == '.'),
    "amount must be a decimal string"
  );
  ensure!(
    value.matches('.').count() <= 1,
    "amount has multiple decimals"
  );
  let mut parts = value.split('.');
  let whole = parts.next().unwrap_or_default();
  let fraction = parts.next();
  ensure!(!whole.is_empty(), "amount is missing whole part");
  ensure!(
    whole.chars().all(|c| c.is_ascii_digit()),
    "invalid whole part"
  );
  if let Some(fraction) = fraction {
    ensure!(
      !fraction.is_empty() && fraction.chars().all(|c| c.is_ascii_digit()),
      "invalid fractional part"
    );
  }
  ensure!(
    value
      .trim_start_matches('0')
      .trim_start_matches('.')
      .chars()
      .any(|c| c != '0'),
    "amount must be greater than zero"
  );
  Ok(value.to_string())
}

fn parse_amount(value: &str, decimals: u8) -> Option<u128> {
  let value = valid_decimal_string(value).ok()?;
  let (whole, fraction) = value.split_once('.').unwrap_or((&value, ""));
  if fraction.len() > decimals.into() {
    return None;
  }

  let scale = 10u128.checked_pow(decimals.into())?;
  let whole = whole.parse::<u128>().ok()?.checked_mul(scale)?;
  let fraction = if fraction.is_empty() {
    0
  } else {
    fraction.parse::<u128>().ok()?.checked_mul(
      10u128
        .checked_pow(decimals.into())?
        .checked_div(10u128.checked_pow(fraction.len().try_into().ok()?)?)?,
    )?
  };

  whole.checked_add(fraction)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn id(byte: u8) -> InscriptionId {
    InscriptionId {
      txid: Txid::from_slice(&[byte; 32]).unwrap(),
      index: 0,
    }
  }

  #[test]
  fn parses_deploy() {
    assert_eq!(
      Payload::from_json_slice(
        br#"{"p":"btcc-20","op":"deploy","tick":"ordi","max":"21000000","lim":"1000","dec":"18"}"#
      )
      .unwrap(),
      Payload {
        protocol: PROTOCOL.into(),
        operation: Operation::Deploy {
          tick: "ordi".into(),
          max: "21000000".into(),
          lim: "1000".into(),
          dec: 18,
        },
      }
    );
  }

  #[test]
  fn parses_mint() {
    assert_eq!(
      Payload::from_json_slice(br#"{"p":"btcc-20","op":"mint","tick":"ordi","amt":"1000"}"#)
        .unwrap(),
      Payload {
        protocol: PROTOCOL.into(),
        operation: Operation::Mint {
          tick: "ordi".into(),
          amt: "1000".into(),
        },
      }
    );
  }

  #[test]
  fn rejects_brc20_protocol() {
    assert!(
      Payload::from_json_slice(br#"{"p":"brc-20","op":"mint","tick":"ordi","amt":"1000"}"#)
        .is_err()
    );
  }

  #[test]
  fn rejects_bad_tick() {
    assert!(
      Payload::from_json_slice(br#"{"p":"btcc-20","op":"mint","tick":"ordix","amt":"1000"}"#)
        .is_err()
    );
  }

  #[test]
  fn ledger_mints_and_transfers_with_brc20_style_two_step_flow() {
    let mut ledger = Ledger::default();

    assert!(
      ledger
        .apply_inscription(
          id(1),
          "alice",
          Payload::from_json_slice(
            br#"{"p":"btcc-20","op":"deploy","tick":"ordi","max":"21000000","lim":"1000","dec":"18"}"#
          )
          .unwrap()
        )
        .valid
    );

    assert!(
      ledger
        .apply_inscription(
          id(2),
          "alice",
          Payload::from_json_slice(br#"{"p":"btcc-20","op":"mint","tick":"ordi","amt":"1000"}"#)
            .unwrap()
        )
        .valid
    );
    assert_eq!(
      ledger.balance("ordi", "alice").available,
      1000_000000000000000000
    );

    assert!(
      ledger
        .apply_inscription(
          id(3),
          "alice",
          Payload::from_json_slice(br#"{"p":"btcc-20","op":"transfer","tick":"ordi","amt":"250"}"#)
            .unwrap()
        )
        .valid
    );
    assert_eq!(
      ledger.balance("ordi", "alice").available,
      750_000000000000000000
    );
    assert_eq!(
      ledger.balance("ordi", "alice").transferable,
      250_000000000000000000
    );

    assert!(ledger.apply_transfer_spend(id(3), "bob").valid);
    assert_eq!(
      ledger.balance("ordi", "alice").available,
      750_000000000000000000
    );
    assert_eq!(ledger.balance("ordi", "alice").transferable, 0);
    assert_eq!(
      ledger.balance("ordi", "bob").available,
      250_000000000000000000
    );
  }

  #[test]
  fn ledger_rejects_mint_over_limit() {
    let mut ledger = Ledger::default();

    ledger.apply_inscription(
      id(1),
      "alice",
      Payload::from_json_slice(
        br#"{"p":"btcc-20","op":"deploy","tick":"ordi","max":"21000000","lim":"1000","dec":"18"}"#,
      )
      .unwrap(),
    );

    assert!(
      !ledger
        .apply_inscription(
          id(2),
          "alice",
          Payload::from_json_slice(br#"{"p":"btcc-20","op":"mint","tick":"ordi","amt":"1001"}"#)
            .unwrap()
        )
        .valid
    );
  }

  #[test]
  fn parses_btcc20_from_ord_witness_inscription() {
    let inscription = Inscription {
      body: Some(br#"{"p":"btcc-20","op":"mint","tick":"ordi","amt":"1000"}"#.to_vec()),
      content_type: Some(b"text/plain;charset=utf-8".to_vec()),
      metaprotocol: Some(PROTOCOL.as_bytes().to_vec()),
      ..default()
    };

    let transaction = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: inscription.to_witness(),
      }],
      output: Vec::new(),
    };

    let envelopes = ParsedEnvelope::from_transaction(&transaction);
    assert_eq!(envelopes.len(), 1);
    assert_eq!(
      Payload::from_inscription(&envelopes[0].payload).unwrap(),
      Some(Payload {
        protocol: PROTOCOL.into(),
        operation: Operation::Mint {
          tick: "ordi".into(),
          amt: "1000".into(),
        },
      })
    );
  }
}
