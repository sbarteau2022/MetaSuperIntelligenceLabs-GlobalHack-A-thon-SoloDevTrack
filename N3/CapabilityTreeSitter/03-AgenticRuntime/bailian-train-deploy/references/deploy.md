# 部署参数参考

> 本文件补充 [`SKILL.md`](../SKILL.md) 第 5 步「创建部署」的 `--plan` 选项、`--model-name` 歧义与计费/运维细则。流程编排与避坑见 SKILL.md。

## --plan 选项（随模型来源不同）

可用 plan **随模型来源不同而不同，不要盲目用 `lora`**：

| 模型来源 | 可用 plan | 说明 |
|---|---|---|
| 链路 A 文本微调输出 | `lora`（默认）/ `mu` | `lora` 按 token 计费，适合验证/低负载 |
| 链路 A 音频 TTS 微调输出 | `mu`（**唯一**） | 不支持 `lora` / `ptu` |
| 链路 A 图像生成微调输出 | `lora`（**唯一**） | 不支持 `mu` / `ptu` |
| 链路 B（基座，如 `qwen3-8b`） | `ptu` / `mu`，**不支持 `lora`** | `ptu` 预留吞吐；`mu` 独占资源 |

各 plan 必填参数：

- `lora`：无额外必填，token 计费，闲置一般不计费（安全默认）。
- `ptu`：需 `--input-tpm` / `--output-tpm`（预留吞吐，闲置也计费）。
- `mu`：需 `--deploy-spec` / `--capacity`（独占资源，闲置也计费）；省略 `--deploy-spec` 时从 `bl deploy models` 自动取。

不确定支持哪些 plan：链路 A 用 `bl deploy models --source custom`，链路 B 用 `bl deploy models --source base`，按返回的 `plans` 选。

## --deploy-spec（mu plan 专用）

`--deploy-spec <spec-id>` 指定部署规格（如 `dps-20260521172224-1vabse`、`MU2`）。省略时 CLI 自动从 `bl deploy models` 查询并按 `billing_method`（默认 POST_PAY）匹配合适的 spec。

> 内部机制：CLI 从 `GET /deployments/models` 响应的 template 对象中读取 `template_id` 字段，在 POST 请求体中作为 `deploy_spec` 发送。CLI flag 为 `--deploy-spec`，不要传 `--template-id`。

## --source 取值（`bl deploy models`）

`custom`（微调输出）/ `base`（基座）/ `public`。

## --model-name 与推理 --model 含义不同（最高频错误，切勿混用）

- `bl deploy <模态> create --model-name` 传的是**导出模型名**（`qwen3-8b-ft-...`，链路 A 来自第 2 步 `finetuned_output`；链路 B 直接传基座名 `qwen3-8b`）。
- 响应里返回的 `output.deployed_model`（如 `qwen3-8b-b98a331831a7`）才是**部署实例 id**。
- 下一步推理（`bl text chat` / `bl speech synthesize` / 异步 API）`--model` 必须用响应里的 `deployed_model`，**不是**你传给 `deploy <模态> create --model-name` 的名字。`--model-name` 与推理 `--model` 指向不同值，不要复用。

## 计费与运维细则

- **闲置计费**：`lora` 按 token 计费，闲置一般不计费，留着无妨；`mu` / `ptu` 是预留资源，闲置也计费，不用要及时清理。
- **删除约束**：`bl deploy delete` 只能删 `STOPPED` / `FAILED` 状态的部署。`RUNNING` 状态的部署先 `bl deploy pause --deployed-model <id>` 停用（mu/ptu 停止计费），再删；或用 `bl deploy delete --deployed-model <id> --skip-precheck` 尝试（跳过本地前置检查，但服务端仍可能拒绝 RUNNING 删除）。暂停后可用 `bl deploy resume --deployed-model <id>` 恢复服务。
- **状态传播延迟**：部署刚到 `RUNNING` 时立即调用可能短暂 404 `Model not exist`，是服务端状态传播延迟，约 1 分钟内稳定，遇 404 等十几秒重试；若持续 404，先核对用的是 `deployed_model` 而非微调输出名。

## 必填 flag（`bl deploy <模态> create`）

- `--model-name`：导出模型名（见上方歧义说明）。
- `--display-name`：控制台显示名。
