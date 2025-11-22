/// Markdown内容状态 - 用于表示内容是否完整
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentState {
    /// 正在输入，语法未完整
    Incomplete,
    /// 语法完整，可以渲染
    Complete,
}
