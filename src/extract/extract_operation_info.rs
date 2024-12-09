#[derive(Default, Clone)]
pub enum OperationStatus {
    #[default]
    Pending,
    Done(Option<u128>),
    Err(String),
}

pub enum OperationStepType {
    Clean,
    Extract,
    Mend,
}

impl OperationStatus {
    pub fn is_done(&self) -> bool {
        matches!(self, OperationStatus::Done(_))
    }

    pub fn cost(&self) -> u128 {
        match self {
            OperationStatus::Done(Some(cost)) => *cost,
            _ => 0,
        }
    }
}

#[derive(Default)]
pub struct ExtractOperationInfo {
    pub clean_state: OperationStatus,

    pub extract_state: OperationStatus,

    pub mend_state: OperationStatus,
}

impl ExtractOperationInfo {
    pub fn all_cost(&self) -> u128 {
        self.clean_state.cost() + self.extract_state.cost() + self.mend_state.cost()
    }

    pub fn is_done(&self) -> bool {
        self.clean_state.is_done() && self.extract_state.is_done() && self.mend_state.is_done()
    }

    pub fn has_error(&self) -> bool {
        matches!(self.clean_state, OperationStatus::Err(_))
            || matches!(self.extract_state, OperationStatus::Err(_))
            || matches!(self.mend_state, OperationStatus::Err(_))
    }

    pub fn first_error_message(&self) -> String {
        if let OperationStatus::Err(msg) = &self.clean_state {
            return msg.clone();
        }
        if let OperationStatus::Err(msg) = &self.extract_state {
            return msg.clone();
        }
        if let OperationStatus::Err(msg) = &self.mend_state {
            return msg.clone();
        }
        "".to_string()
    }
}
