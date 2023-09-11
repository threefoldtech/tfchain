# Validate a runtime

A runtime uploaded to a release is built with [srtool](https://github.com/chevdor/srtool).

To validate such runtime you can use [subwasm](https://github.com/chevdor/subwasm).

## Install subwasm

```sh
cargo install --locked --git https://github.com/chevdor/subwasm --tag v0.19.1
```

Verify installation

`subwasm -v`

## Install srtool

```sh
cargo install --git https://github.com/chevdor/srtool-cli
```

Now build the runtime with srtool:

(change directory to tfchain/substrate-node)

```sh
srtool build --package tfchain-runtime -r runtime --root --verbose
```

It should output something like:

```sh
...
   Compiling tfchain-runtime v2.5.0-rc6 (/build/runtime)
    Finished release [optimized] target(s) in 4m 11s

real    4m11.392s
user    30m13.929s
sys     2m2.417s
✨ Your Substrate WASM Runtime is ready! ✨
Summary generated with srtool v0.11.0 using the docker image paritytech/srtool:1.70.0:
 Package     : tfchain-runtime v2.5.0-rc6
 No GIT information. You are likely running srtool on an archive.
 Rustc       : rustc 1.70.0 (90c541806 2023-05-31)
 Time        : 2023-07-14T07:01:06Z

== Compact
 Version          : substrate-threefold-144 (substrate-threefold-1.tx2.au1)
 Metadata         : V14
 Size             : 2.27 MB (2382577 bytes)
 setCode          : 0xb4d04164310e9fba219e69d761bf7338f9c325ab505b04afbd453ba68109bc2b
 authorizeUpgrade : 0x2055ff679d29f851c30227fbf742622a0044cda26f50a558dfebf9d543ac8cfc
 IPFS             : Qmewd5BgU7EX67NhrVeo1kQzZGw177PCiWxHPZAV2Xr14v
 BLAKE2_256       : 0x09394f8fce643145b1c00eabe0983ee2a9bc7b76339c27742a3955c9f3e80204
 Wasm             : runtime/target/srtool/release/wbuild/tfchain-runtime/tfchain_runtime.compact.wasm

== Compressed
 Version          : substrate-threefold-144 (substrate-threefold-1.tx2.au1)
 Metadata         : V14
 Size             : 487.99 kB (499702 bytes)
 Compression      : 79.03%
 setCode          : 0xd5716602aa617b79a24971afbf424b09fe774f681cc37e6ab10f1495341aed54
 authorizeUpgrade : 0xdaf9e80740ee755b0ae9d5a9c17dd6339f1a911cfe51212551afa817d78f05d3
 IPFS             : QmPzNPVi5vG3UkAXuzQAv9WHjvoQ9jim9NYu2aMPtxRXJ7
 BLAKE2_256       : 0xea2eea0abb04ec6f45ce484b8b7483db941fcd694026dfe2d665de7636643ba8
 Wasm             : runtime/target/srtool/release/wbuild/tfchain-runtime/tfchain_runtime.compact.compressed.wasm
```

## Validate a runtime

First download the runtime from the release page manually.

Now you can validate the runtime with subwasm:

```sh
subwasm info tfchain_runtime.compact.compressed.wasm
```

Should output something like:

```sh
🏋️  Runtime size:             0.477 MB (499,702 bytes)
🗜  Compressed:               Yes, 79.03%
✨ Reserved meta:            OK - [6D, 65, 74, 61]
🎁 Metadata version:         V14
🔥 Core version:             substrate-threefold-144 (substrate-threefold-1.tx2.au1)
🗳️  system.setCode hash:      0xaa970a37172bd074675df488ce2d306e5825c6b5a8aa4cbee3d6129466aa641e
🗳️  authorizeUpgrade hash:    0xdaf9e80740ee755b0ae9d5a9c17dd6339f1a911cfe51212551afa817d78f05d3
🗳️  Blake2-256 hash:          0xea2eea0abb04ec6f45ce484b8b7483db941fcd694026dfe2d665de7636643ba8
📦 IPFS:                     https://www.ipfs.io/ipfs/QmPzNPVi5vG3UkAXuzQAv9WHjvoQ9jim9NYu2aMPtxRXJ7
```

Now compare `Blake2-256 hash` with the one from the srtool output.
