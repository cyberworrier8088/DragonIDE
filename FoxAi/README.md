# FoxAi

A small decoder-only transformer for code auto-completion, written from scratch in Rust on [Burn](https://burn.dev). Build to plug into DragonIDE.

**Status: Not completed yet. work in progress. it does not produce useable completion yet.** The pipeline works to end (data, tokenizer, training, resume, inference). Model quality is the open Probm

## What in here

- Decoder-only transformer: 10 layers, d_model 384, 6 heads, context 512, ~21M params (learned positional embeddings, pre-norm, GELU FFn)
- Byte-level BPE tokenizer written from scratch, 4096 vocab, no external crates. Special tokens: `<pad> <eos> <fim_prefix> <fim_suffix> <fim_middle>`
- Fill-in-the-middle (PSM) training data built at character level, with cuts at places a cursor really sits (after '(', '=', ',', '.', at line starts) plus random cuts
- Corpus cleaning: strips doc comments an d `#[cfg(test)]` modules, drops duplicate 40-line blocks
- Training loop with a time-budget cosine LR schedule, resume, and checkpoints ('foxai', 'best', 'final', 'histroy/')
- Two backends: wgpu (default) and cuda (--features cuda)
- Eval modes: fimtest (30 hand-written prompts), fimeval (excat match on held-out rows)

## Usage

 
```
cargo run --release -- corpus          # collect .rs files into dataset/code.txt
cargo run --release -- tok             # train the tokenizer
cargo run --release -- fimdata 300000  # build FIM training rows
cargo run --release -- train --hours=8 # train (add `resume` to continue)
cargo run --release -- fimtest checkpoints/best.mpk
cargo run --release -- fimeval checkpoints/best.mpk
```

For Cuda add '--features cuda'. Model and tokenizer are a pair: a checkpoint is use less with a different tokenizer.json.

## Where it stands

- Training loss falls normally. Early checkpoints (a few thousand steps) score 0/30 on `fimtest` and 1-2/60 exact match on held-out rows.
- Output is plausible RUst but mostly ignores the code after the cursor.
- Inference is about 25/40 tok/s. No KV cache yet.

## known probms

- The loss covers the whole row (prefix + suffix + middle). only the missle matters for completition. so the model spends most of its capcity on wrong target. Middle-only loss is the next change.
-  The reported FIM val loss is not a real middle-token loss (padding is masked but the mean looks wrongly normlized). Use it only to compare checkpoints.
- Traning data is ~47 MB of Rust from the local cargo registry. Small, and licenses are mixed. Fine for experiments, not for a release.
- The causal mask is rebuilt on the CPU in evary layer on every forward pass. 
- Absolute position embeddings, fixed 512 context.

## Sponsor / support

This is a solo project with no budget. Training so far ran on a free/Pro Colab T4 and a laptop GPU, and that is the main limit: every experiment is about 5H on a T4, and the model and corpus can't grow without more compute.

Looking for a sponsor for training compute (GPU time or credits). It wauld go to:

- longer runs with a bigger model and a larger, license-clean corpus
- re-running the middle-only loss fix and proper eval
- getting FoxAi good enough to replace the external APIs is DragonIDE

If you can help, open an issue on the DragonIDE repo.

## Next

1. Middle-only loss, then fine-tune from the letest checkpoint
2. Real middle-token metric instead of exact match
3. KV cache and a cached causal mask
4. Larger, deduplicated training corpus
5. Tauri command for DragonIDE (trim prefix/suffix to the context budget, return the completion)

## Notes

- Colab link [here](https://colab.research.google.com/drive/11H7POeEOUGNnMNK3H9aZIoN2F8vdD77n?usp=sharing)