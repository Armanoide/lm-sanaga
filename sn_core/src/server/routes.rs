pub enum ApiPath {
    Static(&'static str),
    Dynamic(String),
}

impl ApiPath {
    pub fn as_str(&self) -> &str {
        match self {
            ApiPath::Static(s) => s,
            ApiPath::Dynamic(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BackendApiSession {
    Create,
}

impl BackendApiSession {
    pub fn path(&self) -> ApiPath {
        match self {
            BackendApiSession::Create => ApiPath::Static("/v1/sessions"),
        }
    }
}
#[derive(Debug, Clone)]
pub enum BackendApiMessage {
    Generate,
}

impl BackendApiMessage {
    pub fn path(&self) -> ApiPath {
        match self {
            BackendApiMessage::Generate => ApiPath::Static("/v1/message/generate"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BackendConversationApi {
    List,
}
impl BackendConversationApi {
    pub fn path(&self, session_id: Option<&i32>) -> ApiPath {
        let session_id = session_id
            .map(|i| format!("{}", i))
            .unwrap_or("{session_id}".to_string());
        match self {
            BackendConversationApi::List => {
                ApiPath::Dynamic(format!("/v1/sessions/{}/conversations", session_id))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum BackendApiModel {
    List,
    ListRunning,
    Run,
    Stop,
}

impl BackendApiModel {
    pub fn path(&self) -> ApiPath {
        match self {
            BackendApiModel::List => ApiPath::Static("/v1/models"),
            BackendApiModel::ListRunning => ApiPath::Static("/v1/models/ps"),
            BackendApiModel::Run => ApiPath::Static("/v1/models/run"),
            BackendApiModel::Stop => ApiPath::Static("/v1/models/stop"),
        }
    }
}

pub fn print_all_backend_api_paths() {
    // Sessions
    for session in [BackendApiSession::Create].iter() {
        println!("/api/{}", session.path().as_str());
    }

    // Messages
    for message in [BackendApiMessage::Generate].iter() {
        println!("/api/{}", message.path().as_str());
    }

    // Models
    for model in [
        BackendApiModel::List,
        BackendApiModel::ListRunning,
        BackendApiModel::Run,
        BackendApiModel::Stop,
    ]
    .iter()
    {
        println!("/api/{}", model.path().as_str());
    }

    // Conversations
    for conversation in [BackendConversationApi::List].iter() {
        println!("/api/{}", conversation.path(None).as_str());
    }
}
