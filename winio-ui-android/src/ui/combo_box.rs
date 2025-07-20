use {
    winio_handle::AsWindow,
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct ComboBox;

impl ComboBox {
    pub async fn wait_select(&self) {
        todo!()
    }

    pub fn selection(&self) -> Option<usize> {
        todo!()
    }

    pub fn set_selection(&mut self, _i: Option<usize>) {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }

    pub fn get(&self, _i: usize) -> String {
        todo!()
    }

    pub fn set<S>(&mut self, _i: usize, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn insert<S>(&mut self, _i: usize, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn remove(&mut self, _i: usize) {
        todo!()
    }

    pub fn is_visible(&self) -> bool {
        todo!()
    }

    pub fn set_visible(&mut self, _v: bool) {
        todo!()
    }

    pub fn is_enabled(&self) -> bool {
        todo!()
    }

    pub fn set_enabled(&mut self, _v: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, _p: Point) {
        todo!()
    }

    pub fn size(&self) -> Size {
        todo!()
    }

    pub fn set_size(&mut self, _v: Size) {
        todo!()
    }

    pub fn preferred_size(&self) -> Size {
        todo!()
    }

    pub fn new<W>(_parent: W) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }
}

#[derive(Debug)]
pub struct ComboEntry;

impl ComboEntry {
    pub async fn wait_select(&self) {
        todo!()
    }

    pub async fn wait_change(&self) {
        todo!()
    }

    pub fn text(&self) -> String {
        todo!()
    }

    pub fn set_text<S>(&mut self, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn selection(&self) -> Option<usize> {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn set_selection(&mut self, _i: Option<usize>) {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }

    pub fn get(&self, _i: usize) -> String {
        todo!()
    }

    pub fn set<S>(&mut self, _i: usize, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn insert<S>(&mut self, _i: usize, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn remove(&mut self, _i: usize) {
        todo!()
    }

    pub fn is_visible(&self) -> bool {
        todo!()
    }

    pub fn set_visible(&mut self, _v: bool) {
        todo!()
    }

    pub fn is_enabled(&self) -> bool {
        todo!()
    }

    pub fn set_enabled(&mut self, _v: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point {
        todo!()
    }

    pub fn set_loc(&mut self, _p: Point) {
        todo!()
    }

    pub fn size(&self) -> Size {
        todo!()
    }

    pub fn set_size(&mut self, _v: Size) {
        todo!()
    }

    pub fn preferred_size(&self) -> Size {
        todo!()
    }

    pub fn new<W>(_parent: W) -> Self
    where
        W: AsWindow,
    {
        todo!()
    }
}
