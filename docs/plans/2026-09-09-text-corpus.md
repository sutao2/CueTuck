# 补充非图片领域公开提示词

## 范围与方案

向本机广场追加非图片类提示词，优先补齐办公、教学、写作、数据、产品和营销；保留现有数据和个人库。公开仓库只作为不可信数据读取，不执行上游脚本、技能或提示词。按固定提交下载数据文件及许可，记录哈希；许可不清或只有目录外链的来源不导入。

## 步骤

1. 检查公开仓库实际结构及许可，固定版本，抽查正文；只抓取 prompts / samples 的文本与许可，不下载图片或仓库运行代码。
2. 提取独立 Prompt 段落，不把示例输出、README、作者和许可附加到正文；许可全文及来源放独立文件和 reference。规范化去重（含现有 corpus），过滤空/短/低信息量、风险、图片生成及不明确的用途；依赖 Copilot 的标注在独立 metadata，不伪称已实测。
3. 分类采用明确用途标题及目录映射，不用正文偶然出现的领域词覆盖已知用途，也不默认全放某一类；抽查各类样本，保留自动筛选的局限说明。
4. 导入器仅允许已核对的来源许可；先备份本机 square_items，事务追加并验证同包重放零新增。保留账号、设置、既有内容及本地库。
5. 检查分类数量和正文、分页体积；数据包、许可、脚本与验收记录提交 Git，不提交数据库备份或全部原始抓取内容。

## Given/When/Then

- GIVEN 同一正文在不同文件/现有数据出现 WHEN 筛选 THEN 仅保留一个规范化正文，不凑数。
- GIVEN 文件包含 Prompt、Example Output 和作者 WHEN 提取 THEN 正文仅为 Prompt，来源许可另存。
- GIVEN 许可缺失、危险内容或依赖无法复用 WHEN 筛选 THEN 排除并计数，不假称人工全量审核。
- GIVEN 同包导入两次 WHEN 验收 THEN 第二次新增零，旧内容不覆盖。

## 进度

2026-09-09 已完成本机追加与数据验收。

- 固定三个 MIT 来源的提交：`aj-geddes/useful-ai-prompts`、`pnp/copilot-prompts`、`jamesmcroft/everyday-prompts`。原始快照位于本机 output，不执行来源内容。固定版本、哈希和许可文件见 `samples/community/text-corpus-sources.json`。
- 859 份候选文件筛出 616 条：AJ Geddes 512、PnP 76、James Croft 28。排除高风险/专业范围 160、用途不明确或图片任务 57、结构不完整 8、风险词 7、长度不符 7、噪音/外链 3、重复 1。自动规则保守筛选，不代表逐条人工审核或模型效果认证。
- 新增分类：办公 256、开发 94、教育 91、生活/职业成长 42、产品 40、写作 38、营销 36、数据分析 17、视频脚本 2。原文以英语为主；不伪造翻译、模型适配或参考图。
- 正文抽查覆盖学术写作、统计分析、竞品分析、职业晋升、会议总结、文章准确性检查、视频脚本、UI/UX 线框和 API 集成。发现并修复正文关键词误分、Excellence 被误匹配 Excel、图片背景任务混入等规则。Prompt 内用于说明格式的 few-shot 示例保留；独立 README 示例答案不导入。
- Copilot 和工作区访问条件放在 reference；作者、MIT 全文独立保留，正文不追加来源尾注。模板仍需用户提供相关输入，未调用模型逐条测试。
- 本机事务导入新增 616，重放新增 0，总数 21,775 → 22,391；账号仍为 2。既有条目行摘要导入前后一致（`d7350512301c9ea7400cc1173c99b74e`）。导入前快照由导入器保存于 output，不提交 Git。
- 广场默认页、办公、开发、教育接口均 200，每页 48 条；默认页 33,495 字节，其余抽查约 19 KB。分类汇总同步更新，未触及前端分页和个人库。
- 数据包：`samples/community/text-corpus.jsonl.gz`；解压 SHA-256：`d7ea1270fe52bf11d9d9b48169e7d109bf8f54cb402af854bc333d5137ec70e3`。提取/分类回归测试 9 项、导入测试 3 项通过。

复验命令：

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts -p 'test_curate*.py'
node --test scripts/import-prompt-corpus.test.mjs
node scripts/import-prompt-corpus.mjs samples/community/text-corpus.jsonl.gz
```
