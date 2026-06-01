# BTCC-20

BTCC-20 is this fork's BRC-20-style metaprotocol for BTCC inscriptions.

The inscription envelope stays compatible with Ordinals:

- reveal script envelope: `OP_FALSE OP_IF "ord" ... OP_ENDIF`
- body content type: `text/plain;charset=utf-8`
- metaprotocol: `btcc-20`

The JSON body uses `p:"btcc-20"` and does not accept `p:"brc-20"`:

```json
{"p":"btcc-20","op":"deploy","tick":"cord","max":"21000000000","lim":"1000","dec":"18"}
{"p":"btcc-20","op":"mint","tick":"cord","amt":"1000"}
{"p":"btcc-20","op":"transfer","tick":"cord","amt":"250"}
```

Implemented rule surface:

- `deploy`: first valid ticker wins.
- `mint`: credits available balance to the inscription owner, capped by `lim` and `max`.
- `transfer`: creates a transferable inscription by moving amount from available to transferable balance.
- spending a transfer inscription moves the amount from the old owner to the new owner's available balance.

Useful commands:

```sh
CARGO_TARGET_DIR=/Users/gate/Desktop/codex/btcc/target cargo test btcc20 --lib
CARGO_TARGET_DIR=/Users/gate/Desktop/codex/btcc/target cargo build --bin ord
/Users/gate/Desktop/codex/btcc/target/debug/ord btcc20 decode --help
/Users/gate/Desktop/codex/btcc/target/debug/ord btcc20 scan --start-height 0
```

References:

- Ordinals `ord`: https://github.com/ordinals/ord
- OPI BRC-20 indexer: https://github.com/bestinslot-xyz/OPI
- BRC-20 rule notes: https://docs.bestinslot.xyz/brc-20-indexer-rules
