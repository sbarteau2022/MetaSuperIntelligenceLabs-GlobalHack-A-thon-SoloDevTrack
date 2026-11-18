# 视频生成模型参考

> 本文件补充 [`SKILL.md`](../SKILL.md) 中视频生成微调链路的模态特异性细节（数据格式、模型、超参、部署、推理）。通用流程见 SKILL.md。

## 适用场景

用户想微调一个视频生成模型，用自己的视频数据训练出定制特效、动作或风格的 LoRA 模型。当前支持的基座：

| 基座模型 | 类型 | 说明 |
|---|---|---|
| `wan2.7-i2v` | 首帧图生视频 | 推荐，最新一代，质量最高 |
| `wan2.5-i2v-preview` | 首帧图生视频 | 上一代 |
| `wan2.2-i2v-flash` | 首帧图生视频 | 轻量快速 |
| `wan2.2-kf2v-flash` | 首尾帧图生视频 | 给定首帧+尾帧，生成中间过渡视频 |

## 数据格式

视频生成数据是 **ZIP 包**（不是 .jsonl），内含标注文件和媒体文件。

### 首帧（i2v）数据集

```
i2v-dataset.zip
├── data.jsonl           # 标注文件
├── image_1.jpeg         # 首帧图像（平铺）
├── video_1.mp4          # 训练目标视频
├── image_2.jpeg
└── video_2.mp4
```

JSONL 每行格式：
```json
{"prompt": "视频开头展示了...", "first_frame_path": "image_1.jpg", "video_path": "video_1.mp4"}
```

### 首尾帧（kf2v）数据集

```
kf2v-dataset.zip
├── data.jsonl
├── image/
│   ├── image_1_first.jpg
│   └── image_1_last.jpg
└── video/
    └── video_1.mp4
```

JSONL 每行格式：
```json
{"prompt": "视频开头展示了...", "first_frame_path": "image/image_1_first.jpg", "last_frame_path": "image/image_1_last.jpg", "video_path": "video/video_1.mp4"}
```

### 验证集（可选）

验证集只需首帧图像（kf2v 需首帧+尾帧），**无需视频**。训练时系统会在每个 eval 节点自动生成预览视频。

```json
{"prompt": "视频开头展示了...", "first_frame_path": "image_1.jpg"}
```

### 数据要求

- 图像格式：BMP、JPEG、PNG、WEBP，分辨率 ≤ 4096×4096
- 视频格式：MP4、MOV，分辨率 ≤ 4096×4096
- 视频时长：wan2.2 建议 2~5 秒；wan2.5/2.7 建议 2~10 秒
- 文件名**仅支持英文字符**（ASCII）
- 建议至少 **10 条**数据（20~100 条推荐）
- ZIP 包 ≤ 1GB
- 数据集 schema 为 `video`（CLI 自动探测 `first_frame_path`/`video_path` 字段识别）

### 校验

视频 ZIP **不走** `bl dataset validate`（该命令 `--schema` 仅支持 chatml/dpo/cpt/tts/image）。直接 `bl dataset upload --file <zip-path>`：CLI 依据 `data.jsonl` 里的 `first_frame_path`/`video_path` 字段自动探测 video schema，并在上传时校验 ZIP 内的文件引用。

## 创建训练任务

```bash
bl finetune video create \
  --base-model wan2.7-i2v \
  --datasets <zip-path-or-file-id> \
  --output json
```

### 超参默认值（按模型自动适配）

| 参数 | wan2.7 | wan2.5 | wan2.2 |
|---|---|---|---|
| `batch_size` | 1 | 4 | 4 |
| `max_pixels` | 102400 | 36864 | 262144 |
| `n_epochs` | 50 | 50 | 50 |
| `learning_rate` | 2e-5 | 2e-5 | 2e-5 |
| `eval_epochs` | 20 | 20 | 20 |
| `lora_rank` | 32 | 32 | 32 |
| `lora_alpha` | 32 | 32 | 32 |
| `split` | 0.5 | 0.5 | 0.5 |

用户可通过 `--n-epochs`、`--batch-size`、`--learning-rate` 覆盖默认值。

### 小数据集注意

数据集较小时（< 10 条），默认 `split=0.5` 可能导致验证集 ≤ batch_size 报错。解决：
- 使用 `--batch-size 1` 降低 batch_size
- 或单独上传验证集 `--validations <file-id>`

## 等待训练完成

```bash
bl finetune watch --job-id <job_id> --follow
```

视频训练通常需要**数小时**。训练完成后输出 `finetuned_output`（如 `wan2.7-i2v-ft-202608061949-48ca`），这是部署用的模型名。

## 部署

```bash
bl deploy image create \
  --model-name <finetuned_output> \
  --display-name my-video-lora \
  --plan lora \
  --output json
```

视频 LoRA 部署使用 `plan=lora`（按 token 计费）。部署约 5~10 分钟进入 RUNNING。

```bash
bl deploy get --deployed-model <deployed_model>
```

## 推理与调用

部署 RUNNING 后，用 `bl video generate` 调用：

### 首帧（i2v）— 所有版本

```bash
bl video generate \
  --model <deployed_model> \
  --image https://example.com/first_frame.png \
  --prompt "描述文本" \
  --prompt-extend false
```

### 首尾帧（kf2v）— wan2.2-kf2v

```bash
bl video generate \
  --model <deployed_model> \
  --image https://example.com/first_frame.png \
  --last-frame https://example.com/last_frame.png \
  --prompt "描述文本" \
  --prompt-extend false
```

`--last-frame` 触发 kf2v 模式，自动切换到 `image2video/video-synthesis` 端点。

### 关键参数

| 参数 | 说明 |
|---|---|
| `--prompt-extend false` | LoRA 模型建议关闭 prompt 改写 |
| `--resolution 720P` | 输出分辨率（480P / 720P，wan2.7 支持 1080P） |
| `--download <path>` | 下载视频到本地 |
| `--async` | 仅返回 task_id，不等待 |

## Prompt 编写建议

视频微调的 prompt 结构：`[主体描述] + [背景描述] + [触发词] + [运动描述]`

- **触发词**：无意义稀有词（如 `s86b5p`），作为特效的"视觉锚点"
- **一致性**：所有包含同一特效的样本，运动描述部分保持一致
- **多样性**：主体和场景越丰富越好，避免模型过拟合到单一背景

## 计费

- **训练**：按 token 计费，`bl finetune get --job-id <id>` 查看 `usage_tokens` 和 `training_cost`
- **部署**：plan=lora 免费（闲置不计费）
- **推理调用**：按基础模型标准价格计费
