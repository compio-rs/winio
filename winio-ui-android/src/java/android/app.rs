use super::content::{
    Context, DialogInterfaceOnCancelListener, DialogInterfaceOnClickListener,
    DialogInterfaceOnDismissListener,
};

jni::bind_java_type! {
    pub AlertDialog => android.app.AlertDialog,
    type_map {
        DialogInterfaceOnCancelListener => "android.content.DialogInterface$OnCancelListener",
        DialogInterfaceOnDismissListener => "android.content.DialogInterface$OnDismissListener",
    },
    methods {
        fn show(),
        fn set_on_cancel_listener(listener: &DialogInterfaceOnCancelListener),
        fn set_on_dismiss_listener(listener: &DialogInterfaceOnDismissListener),
    }
}

jni::bind_java_type! {
    pub AlertDialogBuilder => "android.app.AlertDialog$Builder",
    type_map {
        AlertDialog => android.app.AlertDialog,
        Context => android.content.Context,
        DialogInterfaceOnClickListener => "android.content.DialogInterface$OnClickListener",
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn create() -> AlertDialog,
        fn set_message(message: &JCharSequence) -> AlertDialogBuilder,
        fn set_title(title: &JCharSequence) -> AlertDialogBuilder,
        fn set_positive_button(text: &JCharSequence, listener: &DialogInterfaceOnClickListener) -> AlertDialogBuilder,
        fn set_negative_button(text: &JCharSequence, listener: &DialogInterfaceOnClickListener) -> AlertDialogBuilder,
        fn set_neutral_button(text: &JCharSequence, listener: &DialogInterfaceOnClickListener) -> AlertDialogBuilder,
    }
}
