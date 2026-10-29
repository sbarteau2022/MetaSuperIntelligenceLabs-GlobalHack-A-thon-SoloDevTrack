# 音频 TTS 模型参考

> 本文件补充 [`SKILL.md`](../SKILL.md) 中音频 TTS 微调链路的模态特异性细节（数据格式、模型、超参、推理）。通用流程见 SKILL.md。

## 适用场景

用户想微调一个语音合成（TTS）模型，用自己的声音数据训练出定制语音。当前支持的基座：

| 基座模型 | 说明 |
|---|---|
| `cosyvoice-v3-flash` | CosyVoice v3 Flash，中文/多语言 TTS，轻量快速 |

## 数据格式

音频 TTS 数据是 **ZIP 包**（不是 .jsonl），内含：

```
data.zip
├── data.jsonl           # 标注文件，每行 {"wav_fn": "train/xxx.wav", "text": "对应文本"}
└── train/
    ├── sample_001.wav   # 音频文件
    ├── sample_002.wav
    └── ...
```

要求：
- 音频格式 `.wav`，采样率 ≥ 16kHz，时长 2-30s
- `data.jsonl` 中 `wav_fn` 路径必须指向 `train/` 子目录下的文件
- 数据集 schema 为 `tts`（CLI 自动探测 `wav_fn` 字段识别）

### 校验

```bash
bl dataset validate --file <zip-path>             # 自动探测 tts schema
bl dataset validate --file <zip-path> --schema tts  # 显式指定
```

校验内容：ZIP 结构（data.jsonl 存在、train/ 目录、wav 引用合法）+ JSONL 内容（tts schema：每行必须有 wav_fn + text）。

### 生成示例数据

若用户无现成数据，可用 `edge-tts`（Python 库）生成中文语音合成测试数据：

```bash
pip install edge-tts
```

脚本思路：用 `zh-CN-XiaoxiaoNeural` voice 合成 mp3 → ffmpeg 转 16kHz mono wav → 打包 ZIP。效果有限（合成音微调合成音），仅用于跑通流程。

## 创建训练任务

```bash
bl finetune audio create \
  --base-model cosyvoice-v3-flash \
  --datasets <zip-path-or-file-id> \
  --output json
```

要点：
- 音频微调固定用 `sft-lora`（映射到 `efficient_sft`）——`finetune audio create` **不暴露 `--training-type`**，无需也不能传；不支持全参 sft / dpo / cpt
- **无需手动传超参**——8 个音频专有超参（lm_max_epoch、fm_max_epoch 等）由 CLI 自动注入默认值
- `--n-epochs` / `--learning-rate` / `--batch-size` 等文本超参对音频无效，不要传
- capability 检查可能显示 `supports.sft=false`，这是已知矛盾（API 元数据与实际行为不一致），CLI 已自动跳过

从响应记下：`output.job_id`、`output.finetuned_output`（形如 `cosyvoice-v3-flash-ft-<ts>-<id>`）。

## 训练耗时预期

音频 TTS 训练显著慢于文本 SFT。10 条 cosyvoice-v3-flash 样本（3 epoch）约 2.5 小时。样本量增加时耗时线性增长。

## 部署

```bash
bl deploy audio create \
  --model-name <finetuned_output> \
  --display-name <display-name> \
  --plan mu \
  --deploy-spec <spec-id> \
  --capacity 1 \
  --output json
```

要点：
- 音频 TTS 微调模型**只支持 `mu` plan**（不支持 `lora` / `ptu`）
- `--deploy-spec`：从 `bl deploy models --source custom` 查可用 spec，常见值如 `dps-20260521172224-1vabse`（单机部署 MU5）
- `--capacity`：默认 1
- 部署约 6 分钟从 PENDING 到 RUNNING

## 推理与调用

音频 TTS 用 `bl speech synthesize`（不是 `text chat`）：

```bash
bl speech synthesize \
  --model <DEPLOYED_MODEL> \
  --voice default \
  --text "你要合成的文本" \
  --out result.mp3
```

要点：
- `--model` 必须用 `deploy audio create` 响应中的 `deployed_model`（如 `cosyvoice-v3-flash-9d4f3eeb27ca`），**不是** `finetuned_output`
- `--voice` 必须为 `default`（微调模型只有这一个 voice）
- 输出为音频文件（mp3/wav/pcm/opus），用 `--format` 指定
- 推理秒级返回

### 可用参数

`--format`（mp3/pcm/wav/opus）、`--sample-rate`（如 24000）、`--volume`（0-100）、`--rate`（0.5-2.0）、`--pitch`（0.5-2.0）、`--seed`（可复现）、`--language`（zh/en 等）。

## 已知限制

- 仅支持北京区域
- voice 固定 `default`
- capability API 返回 cosyvoice-v3-flash 的 `supports.sft=false`，但 API 实际接受训练——CLI 已自动跳过此检查
- 音频模型不支持 `lora` / `ptu` plan 部署
