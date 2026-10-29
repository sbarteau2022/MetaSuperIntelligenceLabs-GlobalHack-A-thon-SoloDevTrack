# 图像生成模型参考

> 本文件补充 [`SKILL.md`](../SKILL.md) 中图像生成微调链路的模态特异性细节（数据格式、模型、超参、推理）。通用流程见 SKILL.md。

## 适用场景

用户想微调一个图像生成模型，用自己的图像数据训练出定制风格或特定场景的文生图/图生图模型。当前支持的基座：

| 基座模型 | 说明 |
|---|---|
| `wan2.7-image-pro` | Wan2.7 图像生成 Pro 版，质量更高 |
| `wan2.7-image` | Wan2.7 图像生成标准版，速度更快 |

## 数据格式

图像生成数据是 **ZIP 包**（不是 .jsonl），内含：

```
data.zip
├── data.jsonl           # 标注文件
├── img_001.png          # 图像文件（平铺，禁止子目录）
├── img_002.jpg
└── ...
```

**T2I（文生图）** JSONL 每行格式：
```json
{"prompt": "A beautiful landscape", "img_path": "./img_001.png"}
```

**I2I（图生图）** JSONL 每行格式：
```json
{"prompt": "Transform this scene", "input_img": "./in_001.jpg", "img_path": "./out_001.jpg"}
```

要求：
- 图像格式：BMP、JPEG、PNG、WEBP
- 单张 ≤ 20MB，分辨率 ≤ 4096×4096
- 文件名**仅支持英文字符**（ASCII）
- **文件必须平铺放置**，禁止使用子目录（与音频 TTS 的 `train/` 结构不同）
- 建议至少 **25 张**图像（50+ 推荐）
- ZIP 包 ≤ 1GB（vs 文本/音频 300MB）
- 数据集 schema 为 `image`（CLI 自动探测 `img_path` 字段识别）

### 校验

```bash
bl dataset validate --file <zip-path>               # 自动探测 image schema
bl dataset validate --file <zip-path> --schema image  # 显式指定
```

校验内容：ZIP 结构（data.jsonl 存在、≥25 张图像、文件引用合法、扁平结构）+ JSONL 内容（image schema：每行必须有 prompt + img_path，I2I 可选 input_img）。

## 创建训练任务

```bash
bl finetune image create \
  --base-model wan2.7-image-pro \
  --datasets <zip-path-or-file-id> \
  --output json
```

I2I（图生图）模式**自动检测**——只要 JSONL 中有 `input_img` 字段，CLI 自动识别为 I2I 并调整超参（`max_pixels`/`val_img_size` 从 "2k" 切换为 "1k"），无需额外 flag：

```bash
bl finetune image create \
  --base-model wan2.7-image-pro \
  --datasets <i2i-zip-path-or-file-id> \
  --output json
```

要点：
- 图像微调固定用 `sft-lora`（映射到 `efficient_sft`）——`finetune image create` **不暴露 `--training-type`**，无需也不能传；不支持全参 sft / dpo / cpt
- **无需手动传超参**——12 个图像专有超参由 CLI 自动注入默认值
- T2I/I2I 由数据内容自动推断（首行有 `input_img` → I2I，否则 T2I），`max_pixels`/`val_img_size` 随之自动调整
- `--n-epochs` / `--batch-size` 等文本超参对图像无效，不要传
- capability 检查可能显示 `supports.sft=false`，这是已知矛盾，CLI 已自动跳过

从响应记下：`output.job_id`、`output.finetuned_output`（形如 `wan2.7-image-pro-ft-<ts>-<id>`）。

## 训练耗时预期

图像训练耗时取决于 `max_steps`（默认 800 步）。具体时长取决于 GPU 资源和图像分辨率（`max_pixels`）。

## 部署

```bash
bl deploy image create \
  --model-name <finetuned_output> \
  --display-name <display-name> \
  --plan lora \
  --capacity 1 \
  --output json
```

要点：
- 图像生成微调模型**只支持 `lora` plan**（不支持 `mu` / `ptu`）
- `--capacity`：默认 1
- 无需 `--deploy-spec`（lora plan 不需要指定部署规格）

## 推理与调用

图像生成为**异步调用**（不是 `text chat` 或 `speech synthesize`）：

```bash
# 异步提交生成任务
curl -X POST 'https://dashscope.aliyuncs.com/api/v1/services/aigc/image-generation/generation' \
  -H "Authorization: Bearer $DASHSCOPE_API_KEY" \
  -H 'X-DashScope-Async: enable' \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "<DEPLOYED_MODEL>",
    "input": {
      "prompt": "s86b5p a cat sitting on a sofa"
    },
    "parameters": {
      "size": "1024*1024",
      "n": 1
    }
  }'
```

要点：
- `model` 必须用 `deploy image create` 响应中的 `deployed_model`，**不是** `finetuned_output`
- 请求头必须带 `X-DashScope-Async: enable`（异步模式）
- `prompt` 中**必须包含触发词**（训练完成后平台会告知，形如 `s86b5p`）
- 返回 `task_id`，通过轮询获取结果
- 生成的图像 URL 有效期 **24 小时**

### 结果轮询

```bash
# 用返回的 task_id 查询结果
curl -X GET "https://dashscope.aliyuncs.com/api/v1/tasks/<task_id>" \
  -H "Authorization: Bearer $DASHSCOPE_API_KEY"
```

状态流转：`PENDING` → `RUNNING` → `SUCCEEDED`（含图像 URL）

## 已知限制

- 仅支持北京区域
- 文件名仅支持英文字符（ASCII）
- prompt 必须包含训练时分配的触发词
- 图像 URL 24 小时过期
- capability API 返回 wan2.7 系列的 `supports.sft=false`，但 API 实际接受训练——CLI 已自动跳过此检查
- 图像模型不支持 `mu` / `ptu` plan 部署
