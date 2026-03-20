#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
        File,
        Dir,
}

#[derive(Debug, Clone)]
pub struct GithubEntry {
    pub name : String,
    pub path : String,
    pub sha : String,
    pub size : u64,
    pub entry_type : EntryType,
    pub Download_Url : Option<String>,
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub path : String,
    pub sha : String,
    pub size : Option<u64>,
    pub entry_type : EntryType,
}

#[derive(Debug, Clone)]
pub struct GithubError {
    pub message : String,
    pub documentation_url : Option<String>,
}
