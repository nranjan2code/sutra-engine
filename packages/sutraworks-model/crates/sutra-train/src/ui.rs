/// UI state management

#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
    Overview,
    Data,
    Templates,
    Training,
    Results,
    CustomModel,
}

#[derive(Debug, Clone)]
pub struct UIState {
    pub current_tab: TabType,
    pub training_active: bool,
    pub show_help: bool,
    pub show_examples: bool,
    pub show_about: bool,
    pub selected_template: Option<String>,
    pub drag_drop_active: bool,
    pub progress_visible: bool,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            current_tab: TabType::Overview,
            training_active: false,
            show_help: false,
            show_examples: false,
            show_about: false,
            selected_template: None,
            drag_drop_active: false,
            progress_visible: false,
        }
    }
}