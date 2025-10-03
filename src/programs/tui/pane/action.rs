/// Enum for execute an action by pane (used primarily for join the effects of the pane components)
pub enum PaneAction {
    NextFocus,
    PreviousFocus,
    /// Used for blur exclusive_focus and show focus the current element
    RestoreFocus,

    ChangeRequest,
    DeleteRequest,
    ExecuteRequest,
    SetUrlAndMethod,
    SetHeadersAndBody,

    /// Action used for save the project to local
    SaveLocal,
    Noop,
}
