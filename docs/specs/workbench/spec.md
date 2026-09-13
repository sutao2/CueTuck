# 工作台

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M1–M2 实现 |
| 来源 | 原型 `index.html` 主壳 |
| 关联 | [说明：设计源](../../explanation/ui-source.md) |

## Purpose

主窗口的桌面工作台：顶栏、侧栏、内容区、底栏；提供本地库与广场空间。

## Requirements

本轮页面交互以[工作台优化计划](../../plans/2026-09-12-workbench-usability.md)对应场景为准；替代卡片直接编辑、已下载按钮禁用和生产详情排版示例入口。

本轮视觉层级遵循[视觉优化计划](../../plans/2026-09-13-visual-hierarchy.md)：结果标题与数量合入页头，卡片标题和主操作优先，分类计数保留但空间不重复显示本地总数。模型与分类在编辑页紧凑排列，空间不足时换行；其余页面及键盘合同不变。

### Requirement: 内容优先的列表

浏览页 MUST 显示可移除的活动筛选条件；清除仅影响查询、分类和模型，不改变排序与存储数据。本地排序标签计数遵守模型筛选，合集提供明确打开入口。具体场景见[浏览精修计划](../../plans/2026-09-08-browse-refinement.md)。

列表 MUST 以标题、摘要和次要元数据组成紧凑行，不再为「本地」标签单独保留宽列。网格保持统一信息顺序；分类、模型和附件计数可见，长内容截断，操作不能被长标题挤出。

#### Scenario: 文件与长内容展示

- GIVEN 带图片/文件的提示词、合集和超长标题
- WHEN 在窄窗口切换列表/网格及浅深色
- THEN 文件计数无需加载二进制内容，标题/摘要和使用入口保持可读且无横向溢出

### Requirement: 连续操作页面化

工作台连续操作 MUST 使用右侧页面而非模态，包括新建/编辑、使用、合集和广场详情、发布、登录及新建分类，见 [ADR 0020](../../architecture/decisions/0020-workspace-pages.md)。使用一致的返回入口、标题、字段、反馈与操作层级；正文内部滚动。登录使用紧凑单列表单，编辑器元数据窄屏改为单列。设置为应用内完整页面，遵循 ADR 0019；仅必要确认保留对话框。

#### Scenario: 键盘进入与返回页面

- GIVEN 用户从工作台打开登录、编辑、使用、合集、广场详情或发布
- WHEN 用键盘操作页面
- THEN 焦点进入页面，Tab 不强制循环；Escape 走返回或未保存确认逻辑
- AND 返回后恢复到仍存在的入口；输入法组字的 Escape 不离开页面，必要确认仍管理模态焦点

#### Scenario: 导航与父级上下文

- GIVEN 列表带筛选或用户已进入合集/广场详情
- WHEN 编辑、使用、登录后返回，或侧栏切换空间
- THEN 返回保留查询与滚动，成员操作返回原合集；切换侧栏先处理未保存内容和忙碌状态
- AND 只显示一个活动内容页，不让隐藏页面接受键盘焦点

#### Scenario: 所有内容面适配

- GIVEN 浅色或深色桌面，普通或小视口
- WHEN 查看卡片/列表、空态、详情和十个设置页
- THEN 文字、边界、按钮和错误状态保持清晰，长内容不产生页面横向溢出
- AND 不删除现有功能入口，软件内搜索与独立启动器按各自合同工作

#### Scenario: 清楚区分筛选无结果

- GIVEN 当前已使用搜索、分类或模型筛选
- WHEN 没有匹配的条目
- THEN 显示「没有匹配的提示词」并建议调整筛选，不声称整个库为空
- AND 卡片标题本身为键盘可操作的打开入口

### Requirement: 专业桌面视觉

#### Scenario: 全客户端细节一致

- GIVEN 主窗口、设置或独立启动器处于浅色/深色
- WHEN 浏览列表、卡片、表单与反馈
- THEN 使用一致的中性表面和字体层级，主要正文与辅助说明可辨识；输入、选择及按钮的边框与焦点提示统一
- AND 卡片与按钮悬停不改变几何布局，遵循减少动态效果偏好；启动器的原生尺寸、位置和复制粘贴行为不因换肤改变

#### Scenario: 桌面滚动区域与组合控件

- GIVEN 主窗口或独立启动器含超出可视区域的内容
- WHEN 查看浅色/深色下的侧栏、内容区、设置、编辑表单、结果列表或预览并滚动
- THEN 使用共用细圆角滑块与透明轨道；保留滚轮、拖动和键盘滚动，内容与滚动条之间留有间距
- AND 支持的宿主预留稳定滚动槽，内容溢出不使字段宽度跳变；不支持自定义样式的宿主保留原生可用滚动条
- AND 搜索与模型筛选只有外层单一焦点提示，不叠加原生控件描边；模型选项仍支持原生键盘选择

应用身份图标 MUST 使用青绿色底座与白色折纸方舟的同源资源；生成源图与操作记录见 [图标计划](../../plans/2026-09-07-app-icon.md)。桌面 PNG、ICO、ICNS 与启动器品牌图不得各自采用不同标记，不替换功能性线性图标。

#### Scenario: 应用图标一致

- GIVEN 应用打包资源与启动器已加载
- WHEN 查看桌面应用图标或启动器搜索栏
- THEN 使用同一方舟标记，图标透明角保留；启动器品牌占位尺寸和原键盘行为不变

主窗口视觉规则见 [ADR 0017](../../architecture/decisions/0017-screenshot-workbench-frame.md)，设置规则见 [ADR 0019](../../architecture/decisions/0019-settings-page.md)。主窗口使用系统无衬线字体、统一线性图标和中性色表面。空间使用纵向导航，账号与偏好位于侧栏底部；内容工作面与侧栏有明确分区。标题、正文、辅助信息有清晰字号层级；筛选栏和卡片随可用宽度排列，状态栏不抢占内容注意力。

#### Scenario: 截图参考的分栏框架

- GIVEN 主窗口侧栏展开
- WHEN 工作台显示
- THEN 品牌与搜索位于侧栏首行，不占用窗口操作行；内容标题位于顶栏内容侧
- AND 顶栏左区与侧栏同宽，分隔线纵向对齐；内容面没有外边距、圆角外框或阴影
- AND 深色侧栏比内容面稍亮，浅色也保持清晰分区，不改变已保存的主题选择

#### Scenario: 右上角固定搜索入口

- GIVEN 侧栏展开或收起
- WHEN 点击顶栏搜索按钮
- THEN 聚焦当前本地库或广场已有的搜索框，保留当前查询与筛选，不另开搜索窗或唤起启动器
- AND 顶部、侧栏与内容框的提示对应软件内搜索 Cmd+F（macOS）/ Ctrl+F（其他系统），组合键同样聚焦搜索框；弹窗打开或输入法组字时不抢焦点
- AND 右上角不展示设置图标；设置仍可通过侧栏入口或设置快捷键打开，侧栏品牌行搜索保留

#### Scenario: 启动器快捷键标签同步

- GIVEN 用户已保存自定义启动器快捷键
- WHEN 打开主窗口或在设置中重新完成启动器注册与保存
- THEN 底栏「启动器」显示实际保存的宿主格式快捷键，保存后立即更新，无需重启
- AND 未保存的草稿或注册失败不覆盖标签；点击底栏仍唤起原生独立启动器

#### Scenario: 不同视图与窗口下保持一致

- GIVEN 工作台有提示词和合集
- WHEN 切换浅色/深色、网格/列表，或缩窄至 960 像素
- THEN 主操作、导航、内容与反馈保持可辨识，没有页面级横向溢出
- AND 设置占据主窗口，长内容内部滚动，返回时保留工作台；编辑、下载、使用等原动作仍可操作

### Requirement: 壳层结构

#### Scenario: 重启恢复布局与系统主题变化

- GIVEN 用户已调整侧栏宽度、折叠状态或网格/列表视图
- WHEN 工作台重新加载
- THEN 从本地设置恢复有效值，异常值回退默认；首次读取不被默认写入覆盖，拖拽结束再保存宽度
- AND 跟随系统主题实时响应系统变化，手动主题不改变；卸载清理监听

#### Scenario: 连续搜索与分类展开

- GIVEN 用户搜索广场或浏览分类
- WHEN 连续输入或中文组字
- THEN 广场等待 250ms 稳定输入再查询，清空立即查询，组字结束前不查询；本地非组字输入仍即时搜索
- AND 进入广场及显式刷新更新分类/模型字典，连续搜索不重复加载字典，旧结果不能覆盖新查询
- AND 大分类箭头只展开/收起，分类名称只筛选，均可键盘操作

#### Scenario: 拖动侧栏分隔线调宽

- GIVEN 主窗口侧栏展开，默认宽度遵循 ADR 0017
- WHEN 拖动侧栏右边缘，或聚焦分隔条后按左右方向键
- THEN 侧栏与顶栏左区同步调宽，范围为 200–400 CSS 像素，并尽量为内容保留 560 像素；调宽不触发原生窗口拖动
- AND 收起再展开保留当前窗口内的用户宽度，窗口缩窄时限制显示宽度、放大后恢复偏好宽度
- AND 松开、取消、失焦或卸载结束拖动，分隔条提供可读名称和当前宽度

主窗口 MUST 提供侧栏折叠按钮与 `Cmd+B`（macOS）/`Ctrl+B`（其他系统），设置入口另支持 `Cmd+,`/`Ctrl+,`。编辑控件内不拦截侧栏快捷键，弹窗打开时不切换侧栏。

#### Scenario: 侧栏收起与设置快捷键

- GIVEN 主窗口可用且没有编辑弹窗
- WHEN 点击折叠按钮或按侧栏快捷键
- THEN 侧栏隐藏，内容扩展，按钮仍可恢复侧栏
- AND 设置快捷键打开设置，不调用启动器

系统 MUST 提供顶栏、侧栏、内容区、紧凑底栏；保留原型四区，视觉框架以现行 ADR 为准。

#### Scenario: 打开应用

- GIVEN 用户启动桌面应用
- WHEN 主窗口显示
- THEN 可见顶栏当前位置、带品牌的侧栏、内容区与底栏

### Requirement: 空间

系统 MUST 在侧栏顶部提供「提示词广场」与「本地提示词」两个空间入口。广场行为见 [广场规格](../square/spec.md)。侧栏 MUST NOT 显示固定的「本地模式 / 第一期」阶段占位。

#### Scenario: 广场不可用时回到本地

- GIVEN 广场请求失败
- WHEN 用户点击「提示词广场」
- THEN 用户看到明确的离线提示
- AND 可一键回到本地

### Requirement: 侧栏分类

系统 MUST 以「大分类 → 小分类」两级树展示分类，合集不得出现在树上。

#### Scenario: 展开大分类

- GIVEN 「软件开发」下有小分类
- WHEN 用户展开「软件开发」
- THEN 其小分类可见
- AND 合集只出现在内容区

### Requirement: 内容头与视图

系统 MUST 提供搜索、排序或筛选、网格/列表切换。本地主操作是新建，不是发布。

#### Scenario: 切换网格列表

- GIVEN 内容区有至少一条提示词
- WHEN 用户切换到列表视图
- THEN 同一批结果以行展示而不是卡片网格

#### Scenario: 本地最近只含已使用

- GIVEN 本地有一条未使用与一条已使用的提示词
- WHEN 用户打开「最近」
- THEN 只出现已使用的那条，且按 `last_used_at` 新的在前

#### Scenario: 本地收藏只含本机星标

- GIVEN 用户把一条本地提示词标为收藏
- WHEN 用户打开「收藏」
- THEN 只出现已标星的本地条目
- AND 不因此请求广场收藏接口

#### Scenario: 右键只提供已有动作

- GIVEN 内容区有一条本地提示词
- WHEN 用户在卡片上右键
- THEN 菜单含编辑、使用、本机收藏与删除
- AND 不得出现未实现的举报或分享

#### Scenario: 可见更多菜单与多选列表

- GIVEN 本地提示词卡片或列表
- WHEN 点击更多按钮并使用方向键、Home/End 或 Escape
- THEN 菜单与右键共享操作，跳过禁用项；Escape 收起并恢复原按钮焦点
- AND 批量整理时列表行首显示选择框，选中项有描边；切换空间不保留上一空间的操作提示

#### Scenario: 跨页整理与返回位置

- GIVEN 本地提示词超过一页
- WHEN 选择本页、翻页、保存或完成整理
- THEN 选择本页只包含提示词，跨页保留选择；保存与刷新保留页码和滚动，页数缩减时回退到有效末页
- AND 改变筛选或空间时清空选择并回到首页，手动翻页回到内容顶部；整理中禁用翻页与清除筛选

实施与验收见[连续操作计划](../../plans/2026-09-12-workbench-continuity.md)。

#### Scenario: 滚动渲染开销

- GIVEN 长列表与带图页面
- WHEN 滚动但可见行范围未改变
- THEN 虚拟列表不重复渲染卡片，不逐帧重测列样式；非虚拟列表不启动滚动测量
- AND 固定筛选栏采用不透明表面，窗口调宽、追加和详情返回仍维持准确位置

验证范围及限制见[滚动优化计划](../../plans/2026-09-12-scroll-smoothness.md)。

#### Scenario: 长正文的卡片摘要

- GIVEN 正文超过 240 个字符
- WHEN 浏览卡片或行列表
- THEN 卡片 DOM 只包含前 240 个 Unicode 码点及省略号，不拆开代理对
- AND 全文搜索、详情、编辑与复制仍使用完整存储正文，不保存截断文本

实现与验收见[缩略图与摘要计划](../../plans/2026-09-12-thumbnail-cache.md)。

### Requirement: 宿主窗口样式

系统 MUST 按宿主操作系统画窗口控件与快捷键记号。原型只定四区节奏与内容，不定 Windows 风窗框。macOS 上 MUST 使用系统红绿灯（左上），自定义顶栏 MUST 为红绿灯留出 inset，不得把无框矩形窗 + 右侧工具簇当成成品。快捷键展示 MUST 用 Mac 符号（如 `⌃Space`），不得写 `Ctrl Space`。

#### Scenario: macOS 主窗口

- GIVEN 用户在 macOS 打开桌面主窗口
- WHEN 窗口显示
- THEN 左上为系统红绿灯，可拖区域不与按钮重叠
- AND 软件内搜索记号为 `⌘F`，启动器默认记号为 `⌃Space`，自定义组合也用 Mac 符号
- AND 不得出现 Windows 风格的右侧最小化 / 最大化 / 关闭

#### Scenario: 标题栏折叠入口不跳位

- GIVEN 主窗口有系统窗口按钮和侧栏折叠入口
- WHEN 用户反复展开或收起侧栏，或缩窄窗口
- THEN 折叠入口保持相同的左上角坐标，品牌位于侧栏首行，不依赖品牌显隐定位
- AND macOS 顶栏内容从左侧 96 CSS 像素之后开始，系统按钮与应用按钮留有独立操作空间
- AND 折叠按钮本身不是拖动区域，点击只切换侧栏，不启动窗口拖动

### Requirement: 底栏

系统 MUST 显示本地库状态与本地条数，不将本地库就绪写成广场或同步已连接。

#### Scenario: SQLite 就绪

- GIVEN 本地库初始化成功
- WHEN 工作台渲染底栏
- THEN 显示库已就绪与当前未删除的本地提示词数量

## 测试映射

### Requirement: 原生命令参数

客户端 MUST 按 Tauri 的 camelCase 参数协议调用命令，嵌套 API 数据保留 snake_case。浏览器内存测试通过不能替代原生命令参数测试。

#### Scenario: 桌面分类保存

- GIVEN 用户在真实桌面窗口指定分类并保存
- WHEN 调用创建命令
- THEN 分类参数为 categoryId，Rust 收到所选分类
- AND 鉴权命令的 accessToken 与合集的 collectionId 同样正确传入

回归：`platform/tauri.test.js`。

| 场景 | 测试 |
|---|---|
| 全客户端细节一致 | [客户端细化验收](../../plans/2026-09-08-desktop-refinement.md)：浅深色与窗口矩阵、长文本及原生主窗口检查 |
| 重启恢复布局与系统主题变化、连续搜索与分类展开 | `ClientPolish.spec.js` 持久化/无效值、媒体监听、组字/防抖/旧响应、分类独立操作 |
| 键盘进入与返回页面 | `LoginModal.spec.js` 非模态 Tab/归还；`SettingsInteraction.spec.js` 嵌套确认；`UsePromptModal.spec.js` 切步焦点；`CollectionDetailModal.spec.js` 按钮禁用后焦点保留 |
| 页面导航与父级上下文 | `WorkspacePages.spec.js` 查询/滚动保留、侧栏未保存保护、合集成员返回、设置登录草稿保留、保存失败 |
| 所有内容面适配 | Playwright 十页设置 40 组、网格/列表 8 组和编辑器/登录/发布视口测量 |
| 清楚区分筛选无结果 | `WorkbenchShell.spec.js` distinguishes a filtered empty result and exposes a keyboard-operable card title |
| 打开应用 | `desktop/src/components/WorkbenchShell.spec.js` renders four chrome regions；renders prototype sidebar chrome |
| 拖动侧栏分隔线调宽 | `WorkbenchShell.spec.js` pointer capture/bounds/collapse、keyboard/viewport、cancel/blur/unmount；原生桌面 260→340px 后折叠保留 |
| 广场不可用时回到本地 | `desktop/src/components/WorkbenchShell.spec.js` 广场离线与重试场景 |
| SQLite 就绪 | `desktop/src-tauri` `status_is_ready_after_initialize` |
| 展开大分类 | `desktop/src/components/WorkbenchShell.spec.js` loads preset categories into the tree；`lists_children_under_software` |
| 切换网格列表 | `WorkbenchShell.spec.js` shows the same prompts as rows in list view |
| 本地最近只含已使用 | `WorkbenchShell.spec.js` shows recently used local prompts on the recent tab；`library.test.js` records last_used_at when a prompt is used |
| 本地收藏只含本机星标 | `WorkbenchShell.spec.js` shows only starred local prompts on the favorite tab；`localFavorites.test.js` toggles a local favorite id in settings |
| 右键只提供已有动作 | `WorkbenchShell.spec.js` opens a context menu with existing local actions |
| macOS 主窗口 | `WorkbenchShell.spec.js` uses mac chrome on macos；`windowChrome.test.js` gives traffic-light inset and glyph shortcut on macos |
| 标题栏折叠入口不跳位 | `WorkbenchShell.spec.js` keeps the sidebar toggle outside the drag region；Playwright 两态坐标与窄窗口测量 |
| 截图参考框架 / 右上角固定搜索 | `WorkbenchShell.spec.js` 软件内搜索聚焦、两空间与宿主键盘回归；Playwright 点击与真实按键 |
| 启动器快捷键标签同步 | `WorkbenchShell.spec.js` 保存值回读；`SettingsInteraction.spec.js` 保存成功/失败与底栏即时同步 |
| 桌面滚动区域与组合控件 | [滚动条验收计划](../../plans/2026-09-07-scrollbars-focus.md) 浏览器与原生隔离包检查 |
