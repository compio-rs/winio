use std::cell::Cell;

use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_ui_kit::{
    NSLineBreakMode, NSTextAlignment, UIBezierPath, UIColor, UIFont, UILabel, UITraitCollection,
    UIView, UIViewContentMode,
};
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{Result, catch, from_cgsize, from_nsstring, widgets::Widget};

const TITLE_PADDING: f64 = 4.0;
const CONTENT_INSET: f64 = 8.0;
const CORNER_RADIUS: f64 = 8.0;
const BORDER_WIDTH: f64 = 1.0;

#[derive(Debug)]
pub struct GroupBox {
    handle: Widget,
    view: Retained<GroupBoxView>,
    title: Retained<UILabel>,
    content: Retained<UIView>,
    title_height: f64,
}

#[inherit_methods(from = "self.handle")]
impl GroupBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| {
            let view = GroupBoxView::new(mtm);
            view.setOpaque(false);
            view.setContentMode(UIViewContentMode::Redraw);

            let title = UILabel::new(mtm);
            unsafe { title.setTextColor(Some(&UIColor::labelColor())) };
            title.setNumberOfLines(1);
            title.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
            title.setTextAlignment(NSTextAlignment::Natural);
            title.setUserInteractionEnabled(false);

            let font = title
                .font()
                .unwrap_or_else(|| UIFont::systemFontOfSize(UIFont::labelFontSize()));
            let title_height = unsafe { font.lineHeight() }.ceil() + TITLE_PADDING;
            view.set_title_height(title_height);

            let content = UIView::new(mtm);
            content.setClipsToBounds(true);

            view.addSubview(&content);
            view.addSubview(&title);

            let handle = Widget::from_uiview(parent, view.clone().into_super())?;

            let this = Self {
                handle,
                view,
                title,
                content,
                title_height,
            };
            this.update_layout(Size::zero())?;
            Ok(this)
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        self.update_layout(v)?;
        self.view.setNeedsDisplay();
        Ok(())
    }

    pub fn preferred_size(&self) -> Result<Size> {
        catch(|| {
            let width = self.title.intrinsicContentSize().width.max(0.0);
            Size::new(
                width + CONTENT_INSET * 2.0,
                self.title_height + CONTENT_INSET * 2.0,
            )
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        catch(|| from_cgsize(self.content.frame().size))
    }

    pub fn text(&self) -> Result<String> {
        catch(|| {
            self.title
                .text()
                .map(|s| from_nsstring(&s))
                .unwrap_or_default()
        })
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = NSString::from_str(s.as_ref());
        catch(|| self.title.setText(Some(&s)))
    }

    fn update_layout(&self, size: Size) -> Result<()> {
        catch(|| {
            let width = (size.width - CONTENT_INSET * 2.0).max(0.0);
            self.title.setFrame(CGRect::new(
                CGPoint::new(CONTENT_INSET, 0.0),
                CGSize::new(width, self.title_height),
            ));
            let height = (size.height - self.title_height - CONTENT_INSET * 2.0).max(0.0);
            self.content.setFrame(CGRect::new(
                CGPoint::new(CONTENT_INSET, self.title_height + CONTENT_INSET),
                CGSize::new(width, height),
            ));
        })
    }
}

winio_handle::impl_as_widget!(GroupBox, handle);

impl AsContainer for GroupBox {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::ui_kit(&self.content)
    }
}

#[derive(Debug, Default)]
struct GroupBoxViewIvars {
    title_height: Cell<f64>,
}

define_class! {
    #[unsafe(super(UIView))]
    #[name = "WinioGroupBoxViewUIKit"]
    #[ivars = GroupBoxViewIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct GroupBoxView;

    #[allow(non_snake_case)]
    impl GroupBoxView {
        #[unsafe(method_id(initWithFrame:))]
        fn initWithFrame(this: Allocated<Self>, frame: CGRect) -> Option<Retained<Self>> {
            let this = this.set_ivars(GroupBoxViewIvars::default());
            unsafe { msg_send![super(this), initWithFrame: frame] }
        }

        #[unsafe(method(drawRect:))]
        unsafe fn drawRect(&self, _rect: CGRect) {
            let title_height = self.ivars().title_height.get();
            let bounds = self.bounds();
            let rect = CGRect::new(
                CGPoint::new(BORDER_WIDTH / 2.0, title_height + BORDER_WIDTH / 2.0),
                CGSize::new(
                    bounds.size.width - BORDER_WIDTH,
                    bounds.size.height - title_height - BORDER_WIDTH,
                ),
            );
            if rect.size.width <= 0.0 || rect.size.height <= 0.0 {
                return;
            }

            let path =
                UIBezierPath::bezierPathWithRoundedRect_cornerRadius(rect, CORNER_RADIUS);
            UIColor::secondarySystemBackgroundColor().setFill();
            path.fill();
            UIColor::separatorColor().setStroke();
            path.setLineWidth(BORDER_WIDTH);
            path.stroke();
        }

        #[unsafe(method(traitCollectionDidChange:))]
        unsafe fn traitCollectionDidChange(
            &self,
            previous_trait_collection: Option<&UITraitCollection>,
        ) {
            unsafe {
                let () = msg_send![super(self), traitCollectionDidChange: previous_trait_collection];
            };
            self.setNeedsDisplay();
        }
    }
}

impl GroupBoxView {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), initWithFrame: CGRect::ZERO] }
    }

    pub fn set_title_height(&self, height: f64) {
        self.ivars().title_height.set(height);
        self.setNeedsDisplay();
    }
}
