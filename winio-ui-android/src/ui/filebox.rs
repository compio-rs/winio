use {std::path::PathBuf, winio_handle::AsWindow};

#[derive(Debug, Default, Clone)]
pub struct FileBox;

impl FileBox {
    pub fn new() -> Self {
        todo!()
    }

    pub fn title<S>(&mut self, _title: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn filename<S>(&mut self, _filename: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn filters<I>(&mut self, _filters: I)
    where
        I: IntoIterator<Item = FileFilter>,
    {
        todo!()
    }

    pub fn add_filter(&mut self, _filter: FileFilter) {
        todo!()
    }

    pub async fn open<W>(self, _parent: Option<W>) -> Option<PathBuf>
    where
        W: AsWindow,
    {
        todo!()
    }

    pub async fn open_multiple<W>(self, _parent: Option<W>) -> Vec<PathBuf>
    where
        W: AsWindow,
    {
        todo!()
    }

    pub async fn open_folder<W>(self, _parent: Option<W>) -> Option<PathBuf>
    where
        W: AsWindow,
    {
        todo!()
    }

    pub async fn save<W>(self, _parent: Option<W>) -> Option<PathBuf>
    where
        W: AsWindow,
    {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter;

impl FileFilter {
    pub fn new<S>(_name: S, _pattern: S) -> Self
    where
        S: AsRef<str>,
    {
        todo!()
    }
}
