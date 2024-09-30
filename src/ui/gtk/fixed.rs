use gtk4::{glib, prelude::WidgetExt};

mod imp {
    use gtk4::subclass::prelude::*;

    use super::*;

    #[derive(Debug, Default)]
    pub struct Fixed {}

    #[glib::object_subclass]
    impl ObjectSubclass for Fixed {
        type ParentType = gtk4::Fixed;
        type Type = super::Fixed;

        const ABSTRACT: bool = false;
        const NAME: &'static str = "Fixed";
    }

    impl ObjectImpl for Fixed {}

    impl WidgetImpl for Fixed {
        fn measure(&self, orientation: gtk4::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let (m, n) = self.obj().get_size(orientation);
            let n = n.min(for_size);
            (m, n, -1, -1)
        }
    }

    impl FixedImpl for Fixed {}
}

glib::wrapper! {
    pub struct Fixed(ObjectSubclass<imp::Fixed>)
        @extends gtk4::Fixed, gtk4::Widget;
}

impl Default for Fixed {
    fn default() -> Self {
        Self::new()
    }
}

impl Fixed {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn get_size(&self, dir: gtk4::Orientation) -> (i32, i32) {
        let mut nat = 0;
        let mut child = self.first_child();
        while let Some(c) = child {
            if c.get_visible() {
                let alloc = c.size_request();
                match dir {
                    gtk4::Orientation::Horizontal => {
                        let ww = c.preferred_size().1.width();
                        nat = nat.max(alloc.0 + ww);
                    }
                    gtk4::Orientation::Vertical => {
                        let wh = c.preferred_size().1.height();
                        nat = nat.max(alloc.1 + wh);
                    }
                    _ => unreachable!(),
                }
            }
            child = c.next_sibling();
        }
        (0, nat)
    }

    pub fn children(&self) -> Vec<gtk4::Widget> {
        let mut children = vec![];
        let mut child = self.first_child();
        while let Some(c) = child {
            child = c.next_sibling();
            children.push(c);
        }
        children
    }
}
