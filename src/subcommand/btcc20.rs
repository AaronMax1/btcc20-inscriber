use {
  super::*,
  bitcoin::{
    blockdata::{opcodes, script},
    key::UntweakedKeypair,
    secp256k1::{Secp256k1, XOnlyPublicKey, rand},
    taproot::{LeafVersion, TaprootBuilder},
  },
};

#[derive(Debug, Parser)]
pub(crate) enum Btcc20 {
  #[command(about = "Decode BTCC-20 inscriptions from a transaction")]
  Decode(Decode),
  #[command(about = "Inscribe a BTCC-20 deploy, mint, or transfer operation")]
  Inscribe(Inscribe),
  #[command(about = "Create deploy/mint/transfer BTCC-20 inscriptions on regtest")]
  Seed(Seed),
  #[command(about = "Scan blocks for BTCC-20 inscriptions")]
  Scan(Scan),
}

impl Btcc20 {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    match self {
      Self::Decode(decode) => decode.run(settings),
      Self::Inscribe(inscribe) => inscribe.run(settings),
      Self::Seed(seed) => seed.run(settings),
      Self::Scan(scan) => scan.run(settings),
    }
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Inscribe {
  #[arg(
    long,
    default_value = "btcc20",
    help = "Wallet name to create or reuse."
  )]
  wallet: String,
  #[command(subcommand)]
  operation: InscribeOperation,
}

#[derive(Debug, clap::Subcommand)]
enum InscribeOperation {
  Deploy(DeployInscribe),
  Mint(MintInscribe),
  Transfer(TransferInscribe),
}

#[derive(Debug, Parser)]
struct DeployInscribe {
  #[arg(long)]
  tick: String,
  #[arg(long)]
  max: String,
  #[arg(long)]
  lim: String,
  #[arg(long, default_value = "18")]
  dec: u8,
  #[arg(long, help = "Address that will own the deploy inscription.")]
  destination: Option<String>,
}

#[derive(Debug, Parser)]
struct MintInscribe {
  #[arg(long)]
  tick: String,
  #[arg(long)]
  amt: String,
  #[arg(long, help = "Address that will receive the minted balance.")]
  destination: Option<String>,
}

#[derive(Debug, Parser)]
struct TransferInscribe {
  #[arg(long)]
  tick: String,
  #[arg(long)]
  amt: String,
  #[arg(
    long,
    help = "Current owner address that will own the transfer inscription."
  )]
  destination: Option<String>,
}

#[derive(Debug, Parser)]
pub(crate) struct Seed {
  #[arg(
    long,
    default_value = "btcc20-seed",
    help = "Wallet name to create or reuse."
  )]
  wallet: String,
}

#[derive(Debug, Serialize)]
struct SeedOutput {
  deploy: SeedInscription,
  mint: SeedInscription,
  transfer: SeedInscription,
  transfer_spend: Txid,
  scan: ScanOutput,
}

#[derive(Debug, Serialize)]
struct SeedInscription {
  commit: Txid,
  reveal: Txid,
  owner: String,
}

#[derive(Debug, Serialize)]
struct InscribeOutput {
  operation: crate::btcc20::Operation,
  inscription: SeedInscription,
}

#[derive(Debug, Parser)]
pub(crate) struct Decode {
  #[arg(
    long,
    conflicts_with = "file",
    help = "Fetch transaction with <TXID> from Bitcoin Core."
  )]
  txid: Option<Txid>,
  #[arg(long, conflicts_with = "txid", help = "Load transaction from <FILE>.")]
  file: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct Output {
  inscriptions: Vec<DecodedInscription>,
}

#[derive(Debug, Serialize)]
struct DecodedInscription {
  input: u32,
  offset: u32,
  payload: crate::btcc20::Payload,
}

#[derive(Debug, Parser)]
pub(crate) struct Scan {
  #[arg(long, help = "First block height to scan.")]
  start_height: Option<u64>,
  #[arg(long, help = "Last block height to scan. Defaults to the current tip.")]
  end_height: Option<u64>,
  #[arg(long, help = "Resume from a previous btcc20 scan JSON state.")]
  state_in: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ScanOutput {
  start_height: u64,
  end_height: u64,
  blocks: Vec<IndexedBlock>,
  ledger: crate::btcc20::Ledger,
  events: Vec<ScanEvent>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct IndexedBlock {
  height: u64,
  hash: BlockHash,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ScanEvent {
  height: u64,
  txid: Txid,
  inscription: Option<InscriptionId>,
  owner: String,
  valid: bool,
  reason: Option<String>,
  payload: Option<crate::btcc20::Payload>,
}

impl Seed {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    ensure!(
      settings.chain() == Chain::Regtest,
      "btcc20 seed is only available on regtest"
    );

    let root_rpc = BtccRpc::new(&settings, None)?;
    ensure_wallet(&root_rpc, &self.wallet)?;
    let wallet_rpc = BtccRpc::new(&settings, Some(self.wallet.clone()))?;

    let mining_address = wallet_rpc.call::<String>("getnewaddress", Vec::new())?;
    root_rpc.call::<Vec<BlockHash>>(
      "generatetoaddress",
      vec![serde_json::json!(101), serde_json::json!(mining_address)],
    )?;

    let alice = wallet_rpc.call::<String>(
      "getnewaddress",
      vec![serde_json::json!("alice"), serde_json::json!("bech32m")],
    )?;
    let bob = wallet_rpc.call::<String>(
      "getnewaddress",
      vec![serde_json::json!("bob"), serde_json::json!("bech32m")],
    )?;

    let deploy = inscribe(
      &root_rpc,
      &wallet_rpc,
      settings.chain(),
      &alice,
      br#"{"p":"btcc-20","op":"deploy","tick":"ordi","max":"21000000","lim":"1000","dec":"18"}"#,
      true,
    )?;
    let mint = inscribe(
      &root_rpc,
      &wallet_rpc,
      settings.chain(),
      &alice,
      br#"{"p":"btcc-20","op":"mint","tick":"ordi","amt":"1000"}"#,
      true,
    )?;
    let transfer = inscribe(
      &root_rpc,
      &wallet_rpc,
      settings.chain(),
      &alice,
      br#"{"p":"btcc-20","op":"transfer","tick":"ordi","amt":"250"}"#,
      true,
    )?;

    let spend_raw = wallet_rpc.call::<String>(
      "createrawtransaction",
      vec![
        serde_json::json!([{ "txid": transfer.reveal, "vout": 0 }]),
        serde_json::json!({ bob: 0.00098 }),
      ],
    )?;
    let signed = wallet_rpc.call::<SignedRawTransaction>(
      "signrawtransactionwithwallet",
      vec![serde_json::json!(spend_raw)],
    )?;
    ensure!(signed.complete, "wallet did not fully sign transfer spend");
    let transfer_spend =
      wallet_rpc.call::<Txid>("sendrawtransaction", vec![serde_json::json!(signed.hex)])?;
    mine_one(&root_rpc, &wallet_rpc)?;

    let scan = Scan {
      start_height: Some(0),
      end_height: None,
      state_in: None,
    }
    .scan(settings)?;

    Ok(Some(Box::new(SeedOutput {
      deploy,
      mint,
      transfer,
      transfer_spend,
      scan,
    })))
  }
}

impl Inscribe {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let root_rpc = BtccRpc::new(&settings, None)?;
    ensure_wallet(&root_rpc, &self.wallet)?;
    let wallet_rpc = BtccRpc::new(&settings, Some(self.wallet.clone()))?;
    let destination = match &self.operation {
      InscribeOperation::Deploy(deploy) => deploy.destination.clone(),
      InscribeOperation::Mint(mint) => mint.destination.clone(),
      InscribeOperation::Transfer(transfer) => transfer.destination.clone(),
    }
    .map(Ok)
    .unwrap_or_else(|| {
      wallet_rpc.call::<String>(
        "getnewaddress",
        vec![serde_json::json!("btcc20"), serde_json::json!("bech32m")],
      )
    })?;

    let operation = match self.operation {
      InscribeOperation::Deploy(deploy) => crate::btcc20::Operation::Deploy {
        tick: deploy.tick,
        max: deploy.max,
        lim: deploy.lim,
        dec: deploy.dec,
      },
      InscribeOperation::Mint(mint) => crate::btcc20::Operation::Mint {
        tick: mint.tick,
        amt: mint.amt,
      },
      InscribeOperation::Transfer(transfer) => crate::btcc20::Operation::Transfer {
        tick: transfer.tick,
        amt: transfer.amt,
      },
    };
    let body = serde_json::to_vec(&operation_json(&operation))?;
    let inscription = inscribe(
      &root_rpc,
      &wallet_rpc,
      settings.chain(),
      &destination,
      &body,
      settings.chain() == Chain::Regtest,
    )?;

    Ok(Some(Box::new(InscribeOutput {
      operation,
      inscription,
    })))
  }
}

impl Decode {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let transaction = if let Some(txid) = self.txid {
      settings
        .bitcoin_rpc_client(None)?
        .get_raw_transaction(&txid, None)?
    } else if let Some(file) = self.file {
      Transaction::consensus_decode(&mut io::BufReader::new(File::open(file)?))?
    } else {
      Transaction::consensus_decode(&mut io::BufReader::new(io::stdin()))?
    };

    let inscriptions = ParsedEnvelope::from_transaction(&transaction)
      .into_iter()
      .filter_map(|envelope| {
        crate::btcc20::Payload::from_inscription(&envelope.payload)
          .transpose()
          .map(|payload| {
            payload.map(|payload| DecodedInscription {
              input: envelope.input,
              offset: envelope.offset,
              payload,
            })
          })
      })
      .collect::<Result<Vec<_>>>()?;

    Ok(Some(Box::new(Output { inscriptions })))
  }
}

impl Scan {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    Ok(Some(Box::new(self.scan(settings)?)))
  }

  fn scan(self, settings: Settings) -> Result<ScanOutput> {
    let rpc = BtccRpc::new(&settings, None)?;
    let tip = rpc.call::<u64>("getblockcount", Vec::new())?;
    let end_height = self.end_height.unwrap_or(tip);
    let state = if let Some(state_in) = &self.state_in {
      Some(
        serde_json::from_reader::<_, ScanOutput>(io::BufReader::new(File::open(state_in)?))
          .with_context(|| format!("failed to read btcc20 state from {}", state_in.display()))?,
      )
    } else {
      None
    };
    let start_height = self
      .start_height
      .unwrap_or_else(|| state.as_ref().map_or(0, |state| state.end_height + 1));

    ensure!(
      start_height <= end_height || state.is_some(),
      "start height must be <= end height"
    );

    if start_height > end_height {
      return state.context("start height must be <= end height without resume state");
    }

    let mut ledger = state
      .as_ref()
      .map(|state| state.ledger.clone())
      .unwrap_or_default();
    let mut transfer_locations = transfer_locations_from_ledger(&ledger);
    let mut events = state
      .as_ref()
      .map(|state| state.events.clone())
      .unwrap_or_default();
    let mut blocks = state
      .as_ref()
      .map(|state| state.blocks.clone())
      .unwrap_or_default();
    let output_start_height = state
      .as_ref()
      .map_or(start_height, |state| state.start_height);

    if let Some(state) = &state
      && start_height > 0
      && let Some(previous_block) = state
        .blocks
        .iter()
        .find(|block| block.height == start_height - 1)
    {
      let current_hash = rpc.call::<BlockHash>(
        "getblockhash",
        vec![serde_json::json!(previous_block.height)],
      )?;
      ensure!(
        previous_block.hash == current_hash,
        "resume state block hash mismatch at height {}",
        previous_block.height
      );
    }

    for height in start_height..=end_height {
      let hash = rpc.call::<BlockHash>("getblockhash", vec![serde_json::json!(height)])?;
      blocks.push(IndexedBlock { height, hash });
      let block_hex = rpc.call::<String>(
        "getblock",
        vec![serde_json::json!(hash), serde_json::json!(0)],
      )?;
      let block = Block::consensus_decode(&mut Cursor::new(hex::decode(block_hex)?))?;

      for transaction in block.txdata {
        let txid = transaction.compute_txid();

        for input in &transaction.input {
          if let Some(inscription_id) = transfer_locations.remove(&input.previous_output)
            && let Some(owner) = first_output_address(settings.chain(), &transaction)
          {
            let event = ledger.apply_transfer_spend(inscription_id, owner.clone());
            events.push(ScanEvent {
              height,
              txid,
              inscription: Some(inscription_id),
              owner,
              valid: event.valid,
              reason: event.reason,
              payload: None,
            });
          }
        }

        let Some(owner) = first_output_address(settings.chain(), &transaction) else {
          continue;
        };

        for envelope in ParsedEnvelope::from_transaction(&transaction) {
          let inscription_id = InscriptionId {
            txid,
            index: envelope.offset,
          };

          let Some(payload) = crate::btcc20::Payload::from_inscription(&envelope.payload)? else {
            continue;
          };

          let event = ledger.apply_inscription(inscription_id, owner.clone(), payload.clone());

          if event.valid && matches!(payload.operation, crate::btcc20::Operation::Transfer { .. }) {
            transfer_locations.insert(OutPoint { txid, vout: 0 }, inscription_id);
          }

          events.push(ScanEvent {
            height,
            txid,
            inscription: Some(inscription_id),
            owner: owner.clone(),
            valid: event.valid,
            reason: event.reason,
            payload: Some(payload),
          });
        }
      }
    }

    Ok(ScanOutput {
      start_height: output_start_height,
      end_height,
      blocks,
      ledger,
      events,
    })
  }
}

fn transfer_locations_from_ledger(
  ledger: &crate::btcc20::Ledger,
) -> BTreeMap<OutPoint, InscriptionId> {
  ledger
    .transfers
    .iter()
    .filter(|(_inscription_id, transfer)| !transfer.spent)
    .map(|(inscription_id, _transfer)| {
      (
        OutPoint {
          txid: inscription_id.txid,
          vout: 0,
        },
        *inscription_id,
      )
    })
    .collect()
}

fn operation_json(operation: &crate::btcc20::Operation) -> serde_json::Value {
  match operation {
    crate::btcc20::Operation::Deploy {
      tick,
      max,
      lim,
      dec,
    } => serde_json::json!({
      "p": crate::btcc20::PROTOCOL,
      "op": "deploy",
      "tick": tick,
      "max": max,
      "lim": lim,
      "dec": dec.to_string(),
    }),
    crate::btcc20::Operation::Mint { tick, amt } => serde_json::json!({
      "p": crate::btcc20::PROTOCOL,
      "op": "mint",
      "tick": tick,
      "amt": amt,
    }),
    crate::btcc20::Operation::Transfer { tick, amt } => serde_json::json!({
      "p": crate::btcc20::PROTOCOL,
      "op": "transfer",
      "tick": tick,
      "amt": amt,
    }),
  }
}

fn first_output_address(chain: Chain, transaction: &Transaction) -> Option<String> {
  transaction.output.iter().find_map(|output| {
    chain
      .address_from_script(output.script_pubkey.as_script())
      .ok()
      .map(|address| address.to_string())
  })
}

struct BtccRpc {
  url: String,
  username: Option<String>,
  password: Option<String>,
  client: reqwest::blocking::Client,
}

impl BtccRpc {
  fn new(settings: &Settings, wallet: Option<String>) -> Result<Self> {
    let (username, password) = settings.bitcoin_credentials()?.get_user_pass()?;

    Ok(Self {
      url: settings
        .bitcoin_rpc_url(wallet)
        .trim_end_matches('/')
        .to_string(),
      username,
      password,
      client: reqwest::blocking::Client::builder().no_proxy().build()?,
    })
  }

  fn call<T: serde::de::DeserializeOwned>(
    &self,
    method: &str,
    params: Vec<serde_json::Value>,
  ) -> Result<T> {
    let mut request = self.client.post(&self.url).json(&serde_json::json!({
      "jsonrpc": "1.0",
      "id": "btcc20",
      "method": method,
      "params": params,
    }));

    if let Some(username) = &self.username {
      request = request.basic_auth(username, self.password.as_ref());
    }

    let response = request
      .send()
      .with_context(|| format!("failed to call BTCC RPC method `{method}` at {}", self.url))?;
    if let Err(error) = response.error_for_status_ref() {
      let body = response.text().unwrap_or_default();
      bail!(
        "BTCC RPC method `{method}` at {} returned {error}: {body}",
        self.url
      );
    }
    let value: serde_json::Value = response.json()?;
    if !value["error"].is_null() {
      bail!("RPC {method} failed: {}", value["error"]);
    }

    serde_json::from_value(value["result"].clone()).context("failed to decode RPC result")
  }
}

#[derive(Debug, Deserialize)]
struct RpcError {
  code: i64,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
  result: Option<T>,
  error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RawTransactionInfo {
  vout: Vec<RawVout>,
}

#[derive(Debug, Deserialize)]
struct RawVout {
  n: u32,
  #[serde(rename = "scriptPubKey")]
  script_pubkey: RawScriptPubkey,
}

#[derive(Debug, Deserialize)]
struct RawScriptPubkey {
  address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ValidateAddressInfo {
  isvalid: bool,
  #[serde(rename = "scriptPubKey")]
  script_pubkey: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DecodeScriptInfo {
  address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SignedRawTransaction {
  hex: String,
  complete: bool,
}

fn ensure_wallet(root_rpc: &BtccRpc, wallet: &str) -> Result<()> {
  let wallets = root_rpc.call::<Vec<String>>("listwallets", Vec::new())?;
  if wallets.iter().any(|loaded| loaded == wallet) {
    return Ok(());
  }

  match root_rpc
    .call_result::<serde_json::Value>("createwallet", vec![serde_json::json!(wallet)])?
  {
    Ok(_) => Ok(()),
    Err(err) if err.code == -4 || err.code == -35 => {
      root_rpc.call::<serde_json::Value>("loadwallet", vec![serde_json::json!(wallet)])?;
      Ok(())
    }
    Err(err) => bail!("createwallet failed with RPC code {}", err.code),
  }
}

impl BtccRpc {
  fn call_result<T: serde::de::DeserializeOwned>(
    &self,
    method: &str,
    params: Vec<serde_json::Value>,
  ) -> Result<std::result::Result<T, RpcError>> {
    let mut request = self.client.post(&self.url).json(&serde_json::json!({
      "jsonrpc": "1.0",
      "id": "btcc20",
      "method": method,
      "params": params,
    }));

    if let Some(username) = &self.username {
      request = request.basic_auth(username, self.password.as_ref());
    }

    let response = request
      .send()
      .with_context(|| format!("failed to call BTCC RPC method `{method}` at {}", self.url))?;
    if let Err(error) = response.error_for_status_ref() {
      let body = response.text().unwrap_or_default();
      bail!(
        "BTCC RPC method `{method}` at {} returned {error}: {body}",
        self.url
      );
    }
    let response: RpcResponse<T> = response.json()?;
    if let Some(error) = response.error {
      Ok(Err(error))
    } else {
      Ok(Ok(response.result.context("missing RPC result")?))
    }
  }
}

fn inscribe(
  root_rpc: &BtccRpc,
  wallet_rpc: &BtccRpc,
  chain: Chain,
  owner: &str,
  body: &[u8],
  mine: bool,
) -> Result<SeedInscription> {
  let secp = Secp256k1::new();
  let key_pair = UntweakedKeypair::new(&secp, &mut rand::thread_rng());
  let (internal_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);
  let inscription = Inscription {
    body: Some(body.to_vec()),
    content_type: Some(b"text/plain;charset=utf-8".to_vec()),
    metaprotocol: Some(crate::btcc20::PROTOCOL.as_bytes().to_vec()),
    ..default()
  };
  let reveal_script = inscription
    .append_reveal_script_to_builder(script::Builder::new())
    .push_opcode(opcodes::OP_TRUE)
    .into_script();
  let taproot_spend_info = TaprootBuilder::new()
    .add_leaf(0, reveal_script.clone())
    .expect("adding taproot leaf should work")
    .finalize(&secp, internal_key)
    .expect("finalizing taproot builder should work");
  let control_block = taproot_spend_info
    .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
    .expect("computing control block should work");
  let commit_address = Address::p2tr_tweaked(taproot_spend_info.output_key(), chain.network());
  let commit_script_pubkey = commit_address.script_pubkey();
  let commit_address_string = script_address(root_rpc, &commit_script_pubkey)?;

  let commit = wallet_rpc.call::<Txid>(
    "sendtoaddress",
    vec![
      serde_json::json!(commit_address_string),
      serde_json::json!(0.001),
    ],
  )?;
  if mine {
    mine_one(root_rpc, wallet_rpc)?;
  }

  let tx_info = root_rpc.call::<RawTransactionInfo>(
    "getrawtransaction",
    vec![serde_json::json!(commit), serde_json::json!(true)],
  )?;
  let vout = tx_info
    .vout
    .iter()
    .find(|vout| vout.script_pubkey.address.as_deref() == Some(&commit_address_string))
    .context("commit output not found")?
    .n;

  let owner_script_pubkey = address_script_pubkey(root_rpc, owner)?;
  let mut reveal = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint { txid: commit, vout },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(99_000),
      script_pubkey: owner_script_pubkey,
    }],
  };
  reveal.input[0].witness.push(reveal_script);
  reveal.input[0].witness.push(control_block.serialize());

  let reveal = wallet_rpc.call::<Txid>(
    "sendrawtransaction",
    vec![serde_json::json!(consensus::encode::serialize_hex(&reveal))],
  )?;
  if mine {
    mine_one(root_rpc, wallet_rpc)?;
  }

  Ok(SeedInscription {
    commit,
    reveal,
    owner: owner.into(),
  })
}

fn address_script_pubkey(root_rpc: &BtccRpc, address: &str) -> Result<ScriptBuf> {
  let info =
    root_rpc.call::<ValidateAddressInfo>("validateaddress", vec![serde_json::json!(address)])?;
  ensure!(info.isvalid, "invalid BTCC address: {address}");
  let hex = info
    .script_pubkey
    .with_context(|| format!("missing scriptPubKey for BTCC address: {address}"))?;
  Ok(ScriptBuf::from_bytes(hex::decode(hex)?))
}

fn script_address(root_rpc: &BtccRpc, script_pubkey: &ScriptBuf) -> Result<String> {
  let info = root_rpc.call::<DecodeScriptInfo>(
    "decodescript",
    vec![serde_json::json!(hex::encode(script_pubkey.as_bytes()))],
  )?;
  info.address.context("decoded script has no BTCC address")
}

fn mine_one(root_rpc: &BtccRpc, wallet_rpc: &BtccRpc) -> Result<()> {
  let address = wallet_rpc.call::<String>("getnewaddress", Vec::new())?;
  root_rpc.call::<Vec<BlockHash>>(
    "generatetoaddress",
    vec![serde_json::json!(1), serde_json::json!(address)],
  )?;
  Ok(())
}
