# 微调参数参考

> 本文件补充 [`SKILL.md`](../SKILL.md) 第 2 步「创建微调任务」的 `--training-type` 取值与超参细节。流程编排与避坑见 SKILL.md。模态特异性细节（数据格式、模型列表、推理命令）见 [`text.md`](text.md) / [`audio.md`](audio.md) / [`image.md`](image.md)。

## training-type 取值与映射

CLI 用 `<method>` / `<method>-lora` 约定，提交时在 CLI 边界映射到服务端字段（你永远传 CLI 值，不要传服务端字符串）：

| CLI 值 | 服务端 | 适用模态 | 说明 |
|---|---|---|---|
| `sft-lora` | efficient_sft | 文本 / 音频 / 图像 | **默认**，LoRA，便宜快，大多数场景 |
| `sft` | 全参 SFT | 文本 | 效果上限高，成本显著增加 |
| `dpo-lora` / `dpo` | dpo_lora / dpo_full | 文本 | 偏好对齐，需 preference 数据 |
| `cpt` | 继续预训练 | 文本 | 注入领域知识，需非对话格式数据 |

> `cpt` 服务端没有 `-lora` 变体，只有全参；其余方法（sft / dpo）均有 `<method>` 与 `<method>-lora` 两个变体。
> 音频 TTS 和图像生成都只用 `sft-lora`（映射到 `efficient_sft`），不支持其他 training-type；且 `finetune audio create` / `finetune image create` **不暴露 `--training-type` flag**（内部固定 sft-lora），该 flag 仅 `finetune text create` 接受。

## 超参建议

### 文本模型

- `n-epochs` 默认 3。小数据集（几百条）3 够用；过拟合降到 1-2。
- `batch-size` 按数据量自适应（<100KB 自动设 8），一般不手动设。
- `learning-rate` 以**字符串**传入避免 JSON 精度丢失，如 `"1e-4"`。LoRA 默认 `3e-4`（平台对小数据集的设定），过拟合可调小。
- `--validations <path>` 可选，传验证集观察指标。

### 音频 TTS 模型

- 8 个音频专有超参（lm_max_epoch、fm_max_epoch 等）由 CLI 自动注入默认值，无需手动传。
- 文本超参（`n-epochs` / `learning-rate` / `batch-size`）对音频无效，不要传。

### 图像生成模型

- 12 个图像专有超参由 CLI 自动注入默认值，无需手动传。核心参数：
  - `max_steps`（800）、`eval_steps`（200）、`learning_rate`（3e-5）、`lora_rank`（32）
  - `max_pixels` / `val_img_size`：T2I 默认 `"2k"`，I2I 默认 `"1k"`（从数据自动推断，无需手动指定）
  - `generation_type`：从数据自动推断——首行有 `input_img` 字段则为 `"i2i"`，否则 `"t2i"`
  - `split`（0.9）：自动从训练集拆分验证集（仅当未指定 `--validations` 时生效）
- 文本超参（`n-epochs` / `batch-size`）对图像无效，不要传。

## 必填 flag

- `--base-model`：基座模型名（文本如 `qwen3-8b`，音频如 `cosyvoice-v3-flash`，图像如 `wan2.7-image-pro`）。
- `--datasets`：file-id 或本地文件路径（逗号分隔多个）；本地路径会先校验上传再取 file-id。文本传 `.jsonl`，音频/图像传 `.zip`。

## 提交前校验

`bl finetune <模态> create` 提交前会用 listFoundationModels 校验模型是否支持所选 training-type，不支持会快速失败（不耗配额）。若训练集样本数 ≤ batch_size，会在上传/耗配额前被拒。可用 `bl finetune capability --base-model <base>` 提前确认。

> **已知矛盾**：cosyvoice-v3-flash 和 wan2.7 系列的 capability 返回 `supports.sft=false`，但 API 实际接受 `efficient_sft` 训练请求。CLI 已对音频和图像模态自动跳过此检查。
