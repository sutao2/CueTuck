# Mac 主窗口红黄绿按钮对齐

## 方案

1. 读取 Tauri/tao 的原生 inset 实现，并用本机 AppKit 创建不可见窗口测量系统按钮 frame，计算 38px 顶栏的居中偏移。
2. 仅调整主窗口 trafficLightPosition.y；保留横向位置、96px 应用控件留白、顶栏高度、原生按钮与窗口操作。
3. 用相同原生定位算法验证三按钮中心为 19px，运行窗框相关前端回归和配置检查后提交。未发布或替换用户安装应用。

## 场景

- Given Mac 主窗口和 38px 顶栏，When 显示系统按钮，Then 三按钮在顶栏垂直居中，左侧间距保持不变。
- Given Windows 或独立启动器，When 应用本次修复，Then 不改变其窗口配置和行为。

## 验收

- AppKit 不可见原生窗口实测：按钮 height=14、frame.origin.y=9。tao 通过按钮高度加 inset.y 设置标题栏容器高度，原 y=13 导致按钮中心距顶端 11px。改为 y=21 后，三个按钮中心均为 19px，与 38px 顶栏中线一致。
- 按当前配置重放 tao 原生定位算法，三个按钮的横坐标分别为 16/39/62，全部 enabled；修改只涉及 `trafficLightPosition.y`，没有替换系统按钮或修改点击/拖动处理。
- 窗框与工作台 110 项前端测试通过；`cargo check --locked --lib`、文档检查及 diff 空白检查通过。原生检查保留既有 10 项警告，未扩大修改。
- 测量程序和前端日志保留在 `output/verification/macos-traffic-lights/`。未替换用户安装应用、未发布新包；本轮不是安装包点击、全屏切换或多显示器实机验收。
