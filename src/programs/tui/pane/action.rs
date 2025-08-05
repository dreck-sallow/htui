/// Enum for execute an action by pane (used primarily for join the effects of the pane components)
pub enum PaneAction {
    NextFocus,
    PreviousFocus,

    ChangeRequest,
    DeleteRequest,
    ExecuteRequest,
    SetUrlAndMethod,
    SetHeadersAndBody,

    /// Action used for save the project to local
    SaveLocal,
}
